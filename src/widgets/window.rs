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

use crate::chord_name_algorithm::calculate_chord_name;
use crate::chord_ops::prettify_chord_name;
use crate::config::APP_ID;
use crate::database::ChordsDatabase;
use crate::widgets::{FretboardChordDiagram, FretboardChordNameEntry, FretboardChordPreview};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::closure_local;
use gtk::{gio, glib};
use i18n_format::i18n_fmt;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs::File;
use std::path::PathBuf;

const EMPTY_CHORD: [Option<usize>; 6] = [None; 6];
const INITIAL_CHORD_NAME: &str = "C";
const INITIAL_CHORD: [Option<usize>; 6] = [None, Some(3), Some(2), Some(0), Some(1), Some(0)]; // C

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub chord: [Option<usize>; 6],
}

#[derive(Debug, Clone, Copy, Default)]
pub enum GuitarType {
    #[default]
    RightHanded,
    LeftHanded,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type=super::FretboardWindow)]
    #[template(file = "src/widgets/window.blp")]
    pub struct FretboardWindow {
        // Template widgets
        #[template_child]
        pub navigation_stack: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub bookmarks_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub chord_diagram: TemplateChild<FretboardChordDiagram>,
        #[template_child]
        pub entry: TemplateChild<FretboardChordNameEntry>,
        #[template_child]
        pub feedback_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub star_toggle: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub more_variants_button_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub variants_page: TemplateChild<adw::NavigationPage>,
        #[template_child]
        pub variants_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub no_variants_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub variants_scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub variants_container: TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub bookmarks_scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub bookmarks_container: TemplateChild<gtk::FlowBox>,

        pub database: RefCell<ChordsDatabase>,
        pub bookmarks: RefCell<Vec<Bookmark>>,

        #[property(get, set)]
        pub handedness: RefCell<String>,

        pub settings: OnceCell<gio::Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardWindow {
        const NAME: &'static str = "FretboardWindow";
        type Type = super::FretboardWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.install_property_action("win.set-handedness", "handedness");

            klass.install_action("win.empty-chord", None, move |win, _, _| {
                win.empty_chord();
            });
            klass.install_action("win.chord-view", None, move |win, _, _| {
                win.chord_view();
            });
            klass.install_action("win.focus-entry", None, move |win, _, _| {
                win.focus_entry();
            });
            klass.install_action("win.more-variants", None, move |win, _, _| {
                win.more_variants();
            });
            klass.install_action("win.bookmarks", None, move |win, _, _| {
                win.show_bookmarks();
            });
            klass.install_action("win.bookmark-chord", None, move |win, _, _| {
                win.bookmark_chord();
            });
            klass.install_action("win.about", None, move |win, _, _| {
                win.show_about_dialog();
            });

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FretboardWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.set_icon_name(Some(APP_ID));
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
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
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
            .expect("`settings` has been set in `setup_settings`.")
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
        let imp = self.imp();

        self.connect_notify_local(Some("handedness"), move |win, _| {
            let imp = win.imp();

            imp.chord_diagram
                .set_guitar_type(match imp.handedness.borrow().as_str() {
                    "right-handed" => GuitarType::RightHanded,
                    "left-handed" => GuitarType::LeftHanded,
                    other => panic!("unexpected handedness string: {other}"),
                });
        });

        self.settings()
            .bind("handedness", self, "handedness")
            .build();

        let chord_diagram = imp.chord_diagram.get();

        let win: FretboardWindow = self.clone();

        chord_diagram.connect_closure(
            "user-changed-chord",
            false,
            closure_local!(move |_: FretboardChordDiagram| {
                win.load_name_from_chord();
            }),
        );

        let entry = self.imp().entry.get();

        entry.entry().connect_activate(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                win.load_chord_from_name();
            }
        ));

        let win = self.clone();

        entry.connect_closure(
            "enharmonic-clicked",
            false,
            closure_local!(move |_: FretboardChordNameEntry| {
                win.refresh_star_toggle();
            }),
        );

        let star_toggle = self.imp().star_toggle.get();

        star_toggle
            .bind_property("active", &star_toggle, "icon-name")
            .transform_to(|_, active| {
                if active {
                    Some("star-large-symbolic")
                } else {
                    Some("star-outline-rounded-symbolic")
                }
            })
            .sync_create()
            .build();

        star_toggle
            .bind_property("active", &star_toggle, "tooltip-text")
            .transform_to(|_, active| {
                if active {
                    Some(gettext("Remove Bookmark"))
                } else {
                    Some(gettext("Bookmark"))
                }
            })
            .sync_create()
            .build();

        imp.more_variants_button_image
            .set_accessible_role(gtk::AccessibleRole::Presentation);

        imp.no_variants_page.set_description(Some(&i18n_fmt!(
            // translators: The text between the `{}` markers becomes a link to the app's issue tracker. Include both of the markers.
            i18n_fmt("There are no chords with this name in Fretboard’s built-in chord set. If you think there should be, {}reach out{}.", "<a href=\"https://github.com/bragefuglseth/fretboard/issues\">", "</a>")))
        );

        self.load_bookmarks();

        self.refresh_bookmarks_button();

        self.load_stored_chord();
    }

    fn focus_entry(&self) {
        self.set_focus(Some(&self.imp().entry.imp().entry.get()));
    }

    fn bookmark_chord(&self) {
        let imp = self.imp();

        if imp
            .navigation_stack
            .visible_page()
            .expect("stack has a visible page at all times")
            .tag()
            .expect("all pages have tags")
            != "chord-view"
        {
            return;
        }

        let star_toggle = imp.star_toggle.get();
        let current_chord = imp.chord_diagram.imp().chord.get();
        let current_name = imp.entry.serialized_buffer_text();

        self.set_focus(Some(&star_toggle));

        star_toggle.set_active(!star_toggle.is_active());

        let bookmark = Bookmark {
            name: current_name.to_string(),
            chord: current_chord,
        };

        if star_toggle.is_active() {
            self.add_bookmark(bookmark);
        } else {
            self.remove_bookmark(bookmark);
        }

        self.save_bookmarks();
        self.refresh_bookmarks_button();
    }

    fn refresh_bookmarks_button(&self) {
        let button = self.imp().bookmarks_button.get();
        let bookmarks = self.imp().bookmarks.borrow();

        button.set_action_name(if bookmarks.is_empty() {
            None
        } else {
            Some("win.bookmarks")
        });

        let tooltip_text = if bookmarks.is_empty() {
            gettext("No Bookmarks")
        } else {
            gettext("Bookmarks")
        };

        button.set_tooltip_text(Some(&tooltip_text));

        button.set_sensitive(!bookmarks.is_empty());
    }

    fn add_bookmark(&self, bookmark: Bookmark) {
        self.imp().bookmarks.borrow_mut().push(bookmark);
    }

    fn remove_bookmark(&self, query_bookmark: Bookmark) {
        self.imp().bookmarks.replace(
            self.imp()
                .bookmarks
                .take()
                .into_iter()
                .filter(|b| *b != query_bookmark)
                .collect(),
        );
    }

    fn empty_chord(&self) {
        self.imp().chord_diagram.set_chord(EMPTY_CHORD);
        self.imp().entry.overwrite_text("");
        self.imp().feedback_stack.set_visible_child_name("empty");

        self.refresh_star_toggle();
    }

    fn save_current_chord(&self) {
        let chord = self.imp().chord_diagram.imp().chord.get();
        let name = self.imp().entry.serialized_buffer_text();

        let bookmark = Bookmark { name, chord };

        let file = File::create(chord_data_path()).expect("able to create file");
        serde_json::to_writer(file, &bookmark).expect("able to write file");
    }

    fn load_stored_chord(&self) {
        let imp = self.imp();

        let bookmark: Bookmark = if let Ok(file) = File::open(chord_data_path()) {
            serde_json::from_reader(file).expect("able to read file")
        } else {
            Bookmark {
                name: String::from(INITIAL_CHORD_NAME),
                chord: INITIAL_CHORD,
            }
        };

        imp.chord_diagram.set_chord(bookmark.chord);
        imp.entry.overwrite_text(&bookmark.name);

        if bookmark.chord != EMPTY_CHORD {
            imp.feedback_stack.set_visible_child_name("chord-actions");
        }

        self.refresh_star_toggle();
    }

    fn save_bookmarks(&self) {
        let bookmarks = self.imp().bookmarks.borrow();

        let file = File::create(bookmarks_data_path()).expect("able to create file");
        serde_json::to_writer(file, &*bookmarks).expect("able to write file");
    }

    fn load_bookmarks(&self) {
        let bookmarks: Vec<Bookmark> = if let Ok(file) = File::open(bookmarks_data_path()) {
            serde_json::from_reader(file).expect("able to read file")
        } else {
            Vec::new()
        };

        self.imp().bookmarks.replace(bookmarks);
    }

    fn load_chord_from_name(&self) {
        let imp = self.imp();

        let name = imp.entry.serialized_buffer_text();
        let db = imp.database.borrow();
        let chord_opt = db.chord_from_name(&name).and_then(|c| c.positions.first());

        if let Some(chord) = chord_opt {
            imp.chord_diagram.set_chord(*chord);
            imp.feedback_stack.set_visible_child_name("chord-actions");
        } else {
            imp.chord_diagram.set_chord(EMPTY_CHORD);
            imp.feedback_stack.set_visible_child_name("label");
        }

        self.refresh_star_toggle();
    }

    fn load_name_from_chord(&self) {
        let imp = self.imp();

        let query_chord = imp.chord_diagram.imp().chord.get();

        let name_opt = imp.database.borrow().name_from_chord(query_chord);

        let name =
            name_opt.unwrap_or_else(|| calculate_chord_name(query_chord).unwrap_or_default());

        imp.entry.overwrite_text(&name);
        imp.feedback_stack
            .set_visible_child_name(if !name.is_empty() {
                "chord-actions"
            } else {
                "label"
            });

        self.refresh_star_toggle();
    }

    fn refresh_star_toggle(&self) {
        let imp = self.imp();

        let chord = imp.chord_diagram.get().imp().chord.get();
        let name = imp.entry.serialized_buffer_text();

        let query = Bookmark { name, chord };

        let star_toggle = imp.star_toggle.get();

        let bookmarks = imp.bookmarks.borrow();

        star_toggle.set_active(bookmarks.iter().any(|bm| bm == &query))
    }

    fn chord_view(&self) {
        self.imp().navigation_stack.push_by_tag("chord-view");
    }

    fn more_variants(&self) {
        let imp = self.imp();

        let chord_name = imp.entry.serialized_buffer_text();

        if chord_name.is_empty() {
            return;
        }
        if imp
            .navigation_stack
            .visible_page()
            .expect("stack has a visible page at all times")
            .tag()
            .expect("all pages have tags")
            != "chord-view"
        {
            return;
        }

        let db = imp.database.borrow();

        let variants = db
            .chord_from_name(&chord_name)
            .map(|chord| &chord.positions)
            .cloned()
            .unwrap_or_default();

        imp.variants_stack
            .set_visible_child_name(if variants.is_empty() {
                "no-variants"
            } else {
                "variants"
            });

        let container = imp.variants_container.get();
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        for variant in variants {
            let preview = FretboardChordPreview::with_chord(
                variant,
                imp.chord_diagram.imp().guitar_type.get(),
            );
            let buffer = imp.entry.serialized_buffer_text();
            preview.imp().chord_name.replace(buffer);

            let button = gtk::Button::builder()
                .css_classes(["flat", "fretboard-chord-preview-button"])
                .halign(gtk::Align::Center)
                .child(&preview)
                .build();

            button.connect_clicked(glib::clone!(
                #[weak(rename_to = win)]
                self,
                #[weak]
                preview,
                move |_| {
                    let name = preview.imp().chord_name.borrow();
                    let chord = preview.imp().chord.get();

                    win.imp().chord_diagram.set_chord(chord);
                    win.imp().entry.overwrite_text(&name);
                    win.refresh_star_toggle();
                    win.imp().navigation_stack.pop();
                }
            ));

            // We already have a button, so having the FlowBoxChild focusable would create an
            // unnecessary layer of indirection
            let flow_box_child = gtk::FlowBoxChild::builder()
                .focusable(false)
                .child(&button)
                .build();

            container.insert(&flow_box_child, -1);
        }

        imp.variants_page
            .set_title(&prettify_chord_name(&chord_name));
        imp.variants_scrolled_window
            .set_vadjustment(Some(&gtk::Adjustment::builder().lower(0.0).build()));

        self.imp().navigation_stack.push_by_tag("more-variants");
    }

    fn show_bookmarks(&self) {
        let imp = self.imp();

        if imp
            .navigation_stack
            .visible_page()
            .expect("stack has a visible page at all times")
            .tag()
            .expect("all pages have tags")
            != "chord-view"
        {
            return;
        }
        if imp.bookmarks.borrow().is_empty() {
            return;
        }

        let container = imp.bookmarks_container.get();

        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let bookmarks = imp.bookmarks.borrow();

        for bookmark in bookmarks.iter().rev() {
            let preview = FretboardChordPreview::with_chord(
                bookmark.chord,
                imp.chord_diagram.imp().guitar_type.get(),
            );
            preview.imp().chord_name.replace(bookmark.name.clone());

            let label = gtk::Label::builder()
                .label(&prettify_chord_name(&bookmark.name))
                .justify(gtk::Justification::Center)
                .css_classes(["title-3"])
                .build();

            let bookmark_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(12)
                .build();

            bookmark_box.append(&preview);
            bookmark_box.append(&label);

            let button = gtk::Button::builder()
                .css_classes(["flat", "fretboard-chord-preview-button"])
                .hexpand(false)
                .child(&bookmark_box)
                .build();

            button.connect_clicked(glib::clone!(
                #[weak(rename_to = win)]
                self,
                #[weak]
                preview,
                move |_| {
                    let name = preview.imp().chord_name.borrow();
                    let chord = preview.imp().chord.get();

                    win.imp().chord_diagram.set_chord(chord);
                    win.imp().entry.overwrite_text(&prettify_chord_name(&name));
                    win.refresh_star_toggle();
                    win.imp().navigation_stack.pop();
                }
            ));

            // We already have a button, so having the FlowBoxChild focusable would create an
            // unnecessary layer of indirection
            let flow_box_child = gtk::FlowBoxChild::builder()
                .focusable(false)
                .child(&button)
                .build();

            container.insert(&flow_box_child, -1);
        }

        imp.bookmarks_scrolled_window
            .set_vadjustment(Some(&gtk::Adjustment::builder().lower(0.0).build()));

        imp.navigation_stack.push_by_tag("bookmarks");
    }

    fn show_about_dialog(&self) {
        let about = adw::AboutDialog::from_appdata(
            "/dev/bragefuglseth/Fretboard/dev.bragefuglseth.Fretboard.metainfo.xml",
            Some("8.0"),
        );

        about.set_developers(&["Brage Fuglseth https://bragefuglseth.dev"]);
        about.set_copyright("© 2024 Brage Fuglseth");
        // Translators: Replace "translator-credits" with your names, one name per line
        about.set_translator_credits(&gettext("translator-credits"));

        about.add_other_app(
            "dev.bragefuglseth.Keypunch",
            // Translators: Metainfo for the app Keypunch. <https://github.com/bragefuglseth/keypunch>
            &gettext("Keypunch"),
            // Translators: Metainfo for the app Keypunch. <https://github.com/bragefuglseth/keypunch>
            &gettext("Practice your typing skills"),
        );

        about.present(Some(
            self.root().unwrap().downcast_ref::<gtk::Window>().unwrap(),
        ));
    }
}

fn chord_data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path).expect("able to create directory");
    path.push("current_chord.json");
    path
}

fn bookmarks_data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path).expect("able to create directory");
    path.push("bookmarks.json");
    path
}
