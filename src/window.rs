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

use crate::barre_spin::FretboardBarreSpin;
use crate::chord_diagram::FretboardChordDiagram;
use crate::chords::{load_chords, Chord};
use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};
use rayon::prelude::*;
use std::cell::RefCell;

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
        pub barre_spin: TemplateChild<FretboardBarreSpin>,

        pub chords: RefCell<Vec<Chord>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardWindow {
        const NAME: &'static str = "FretboardWindow";
        type Type = super::FretboardWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().init();
        }
    }
    impl WidgetImpl for FretboardWindow {}
    impl WindowImpl for FretboardWindow {}
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

        let barre_spin = self.imp().barre_spin.get();
        let chord_diagram = self.imp().chord_diagram.get();

        // barre_spin.connect_value_notify(glib::clone!(@weak chord_diagram => move |val| {
        //     chord_diagram.update_neck_position(val);
        // }));

        self.setup_chords();
    }

    fn setup_chords(&self) {
        self.imp().chords.replace(load_chords());

        let chords = self.imp().chords.borrow();
        let chord = chords
            .par_iter()
            .find_first(|chord| chord.name.to_lowercase() == "F".to_lowercase())
            .map(|chord| chord.positions[0].clone())
            .unwrap();

        self.imp().chord_diagram.set_chord(chord);
        self.imp()
            .barre_spin
            .set_value(self.imp().chord_diagram.neck_position());
    }
}
