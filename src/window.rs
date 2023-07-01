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
    chord_diagram::FretboardChordDiagram,
    chord_name_entry::FretboardChordNameEntry,
    chords::{load_chords, Chord},
    config::{APP_ID},
};
use adw::subclass::prelude::*;
use glib::{signal::Inhibit, closure_local};
use gtk::prelude::*;
use gtk::{gio, glib};
use rayon::prelude::*;
use std::cell::RefCell;
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::fs::File;

const EMPTY_CHORD: [Option<usize>; 6] = [None; 6];
const INITIAL_CHORD: [Option<usize>; 6] = [None, Some(3), Some(2), Some(0), Some(1), Some(0)]; // C

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Fretboard/window.ui")]
    pub struct FretboardWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub filler: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub chord_diagram: TemplateChild<FretboardChordDiagram>,
        #[template_child]
        pub entry: TemplateChild<FretboardChordNameEntry>,
        #[template_child]
        pub feedback_stack: TemplateChild<gtk::Stack>,

        pub chords: RefCell<Vec<Chord>>,

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
        fn close_request(&self) -> Inhibit {
            // Save window size
            self.obj()
                .save_window_size()
                .expect("able to save window state");

            self.obj().save_current_chord();

            // Don't inhibit the default handler
            self.parent_close_request()
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
        // on narrow window width, hide filler
        self.bind_property("default-width", &self.imp().filler.get(), "reveal-child")
            .transform_to(|_, window_width: i32| {
                if window_width > 420 {
                    Some(true)
                } else {
                    Some(false)
                }
            })
            .sync_create()
            .build();

        let chord_diagram = self.imp().chord_diagram.get();

        let win: FretboardWindow = self.clone();

        chord_diagram.connect_closure(
            "user-changed-chord",
            false,
            closure_local!(move |_: FretboardChordDiagram| {
                win.lookup_chord_name();
            }),
        );

        let entry = self.imp().entry.get();

        entry
            .entry()
            .connect_activate(glib::clone!(@weak self as win => move |entry| {
                win.load_chord_from_name(&entry.text());
                win.imp().entry.get().imp().entry_buffer.replace(entry.text().as_str().to_string());
            }));

        // load chords
        self.imp().chords.replace(load_chords());
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
        serde_json::to_writer(file, &chord)
            .expect("able to write file");
    }

    fn load_stored_chord(&self) {
        let chord: [Option<usize>; 6] = if let Ok(file) = File::open(data_path()) {
            serde_json::from_reader(file).expect("able to read file")
        } else {
            INITIAL_CHORD
        };

        self.imp().chord_diagram.set_chord(chord);
        self.lookup_chord_name();
    }

    fn load_chord_from_name(&self, name: &str) {
        let chords = self.imp().chords.borrow();
        let chord_opt = chords
            .par_iter()
            .find_first(|chord| chord.name.to_lowercase() == name.to_lowercase())
            .map(|chord| chord.positions[0].clone());

        if let Some(chord) = chord_opt {
            self.imp().chord_diagram.set_chord(chord);
            self.imp().feedback_stack.set_visible_child_name("empty");
        } else {
            self.imp().chord_diagram.set_chord(EMPTY_CHORD);
            self.imp().feedback_stack.set_visible_child_name("label");
        }
    }

    fn lookup_chord_name(&self) {
        let query_chord = self.imp().chord_diagram.imp().chord.get();

        let chords = self.imp().chords.borrow();
        let name_opt = chords
            .par_iter()
            .find_first(|chord| {
                chord
                    .positions
                    .par_iter()
                    .any(|&position| position == query_chord)
            })
            .map(|chord| chord.name.to_owned());

        if let Some(name) = name_opt {
            self.imp()
                .entry
                .imp()
                .entry_buffer
                .replace(name.to_string());
            self.imp().entry.entry().set_text(&name);
            self.imp().feedback_stack.set_visible_child_name("empty");
        } else {
            self.imp()
                .entry
                .imp()
                .entry_buffer
                .replace(String::from(""));
            self.imp().entry.entry().set_text("");
            self.imp().feedback_stack.set_visible_child_name("label");
        }
    }
}

fn data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push("dev.bragefuglseth.Fretboard");
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    path.push("chord.json");
    path
}
