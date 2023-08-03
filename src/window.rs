/* window.rs
 *
 * Copyright 2023 Brage Fuglseth
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::{
    chord_diagram::FretboardChordDiagram, chord_name_algorithm::calculate_chord_name,
    chord_name_entry::FretboardChordNameEntry, chord_preview::FretboardChordPreview,
    config::APP_ID, database::ChordsDatabase,
};
use adw::subclass::prelude::*;
use glib::closure_local;
use gtk::prelude::*;
use gtk::{gio, glib};
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::fs::File;
use std::path::PathBuf;

const EMPTY_CHORD: [Option<usize>; 6] = [None; 6];
const INITIAL_CHORD: [Option<usize>; 6] = [None, Some(3), Some(2), Some(0), Some(1), Some(0)]; // C

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Fretboard/window.ui")]
    pub struct FretboardWindow {
        // Template widgets
        #[template_child]
        pub navigation_stack: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub chord_diagram: TemplateChild<FretboardChordDiagram>,
        #[template_child]
        pub entry: TemplateChild<FretboardChordNameEntry>,
        #[template_child]
        pub feedback_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub variants_window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub variants_scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub variants_container: TemplateChild<gtk::FlowBox>,

        pub database: RefCell<ChordsDatabase>,

        pub settings: OnceCell<gio::Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardWindow {
        const NAME: &'static str = "FretboardWindow";
        type Type = super::FretboardWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("win.empty-chord", None, move |win, _, _| {
                win.empty_chord();
            });
            klass.install_action("win.chord-view", None, move |win, _, _| {
                win.chord_view();
            });
            klass.install_action("win.more-variants", None, move |win, _, _| {
                win.more_variants();
            });

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.setup_settings();
            obj.load_window_size();

            if APP_ID.ends_with("Devel") {
                obj.add_css_class("devel");
            }

            obj.init();
        }
    }
    impl WidgetImpl for FretboardWindow {}
    impl WindowImpl for FretboardWindow {
        fn close_request(&self) -> glib::Propagation {
            // Save window size
            self.obj()
                .save_window_size()
                .expect("able to save window state");

            self.obj().save_current_chord();

            // Don't inhibit the default handler
            glib::Propagation::Proceed
        }
    }
    impl ApplicationWindowImpl for FretboardWindow {}
    impl AdwApplicationWindowImpl for FretboardWindow {}
}

glib::wrapper! {
    pub struct FretboardWindow(ObjectSubclass<imp::FretboardWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl FretboardWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_settings(&self) {
        let settings = gio::Settings::new("dev.bragefuglseth.Fretboard");
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` has not been set");
    }

    fn settings(&self) -> &gio::Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` have been set in `setup_settings`.")
    }

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        // Get the size of the window
        let size = self.default_size();

        // Set the window state in `settings`
        self.settings().set_int("window-width", size.0)?;
        self.settings().set_int("window-height", size.1)?;
        self.settings()
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let width = self.settings().int("window-width");
        let height = self.settings().int("window-height");
        let is_maximized = self.settings().boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    fn init(&self) {
        let chord_diagram = self.imp().chord_diagram.get();

        let win: FretboardWindow = self.clone();

        chord_diagram.connect_closure(
            "user-changed-chord",
            false,
            closure_local!(move |_: FretboardChordDiagram| {
                win.load_name_from_chord();
            }),
        );

        let entry = self.imp().entry.get();

        entry
            .entry()
            .connect_activate(glib::clone!(@weak self as win => move |entry| {
                win.load_chord_from_name();
                win.imp().entry.get().imp().entry_buffer.replace(entry.text().as_str().to_string());
            }));

        self.load_stored_chord();
    }

    fn empty_chord(&self) {
        self.imp().chord_diagram.set_chord(EMPTY_CHORD);
        self.imp().entry.imp().entry.set_text("");
        self.imp().feedback_stack.set_visible_child_name("empty");
    }

    fn save_current_chord(&self) {
        let chord = &self.imp().chord_diagram.imp().chord.get();

        let file = File::create(data_path()).expect("able to create file");
        serde_json::to_writer(file, &chord).expect("able to write file");
    }

    fn load_stored_chord(&self) {
        let chord: [Option<usize>; 6] = if let Ok(file) = File::open(data_path()) {
            serde_json::from_reader(file).expect("able to read file")
        } else {
            INITIAL_CHORD
        };

        self.imp().chord_diagram.set_chord(chord);
        self.load_name_from_chord();
    }

    fn load_chord_from_name(&self) {
        let name = self.imp().entry.get().imp().entry.text().to_string();
        let db = self.imp().database.borrow();
        let chord_opt = db
            .chord_from_name(&name)
            .map(|c| c.positions.get(0).unwrap());

        if let Some(chord) = chord_opt {
            self.imp().chord_diagram.set_chord(*chord);
            self.imp()
                .feedback_stack
                .set_visible_child_name("chord-actions");
        } else {
            self.imp().chord_diagram.set_chord(EMPTY_CHORD);
            self.imp().feedback_stack.set_visible_child_name("label");
        }
    }

    fn load_name_from_chord(&self) {
        let query_chord = self.imp().chord_diagram.imp().chord.get();

        let name_opt = self.imp().database.borrow().name_from_chord(query_chord);

        let name = if let Some(name) = name_opt {
            name
        } else if let Some(name) = calculate_chord_name(query_chord) {
            name
        } else {
            Default::default()
        };

        self.imp().entry.imp().entry_buffer.replace(name.clone());
        self.imp().entry.entry().set_text(&name);
        self.imp()
            .feedback_stack
            .set_visible_child_name(if !name.is_empty() {
                "chord-actions"
            } else {
                "label"
            });
    }

    fn chord_view(&self) {
        self.imp()
            .navigation_stack
            .set_visible_child_name("chord-view");
    }

    fn more_variants(&self) {
        let imp = self.imp();
        let chord_name = imp.entry.imp().entry_buffer.borrow();


        let db = imp.database.borrow();

        let variants = db
            .chord_from_name(&chord_name)
            .map(|chord| &chord.positions)
            .cloned()
            .unwrap_or_else(|| Vec::new());

        let var_con = imp.variants_container.get();
        while let Some(child) = var_con.first_child() {
            var_con.remove(&child);
        }

        for variant in variants {
            let preview = FretboardChordPreview::with_chord(variant);

            preview.button().connect_clicked(
                glib::clone!(@weak self as win, @weak preview => move |_| {
                    let chord = preview.imp().chord.get();
                    win.imp().chord_diagram.set_chord(chord);
                    win.chord_view();
                }),
            );

            var_con.insert(&preview, -1);
        }

        imp.variants_window_title.set_title(&chord_name);
        imp.variants_scrolled_window
            .set_vadjustment(Some(&gtk::Adjustment::builder().lower(0.0).build()));

        self.imp()
            .navigation_stack
            .set_visible_child_name("more-variants");
    }
}

fn data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push("dev.bragefuglseth.Fretboard");
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    path.push("chord.json");
    path
}
