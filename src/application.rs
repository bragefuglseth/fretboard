/* application.rs
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

use crate::FretboardWindow;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use adw::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FretboardApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardApplication {
        const NAME: &'static str = "FretboardApplication";
        type Type = super::FretboardApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for FretboardApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_resource_base_path(Some("/dev/bragefuglseth/Fretboard/"));
        }
    }

    impl ApplicationImpl for FretboardApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = FretboardWindow::new(&*application);
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for FretboardApplication {}
    impl AdwApplicationImpl for FretboardApplication {}
}

glib::wrapper! {
    pub struct FretboardApplication(ObjectSubclass<imp::FretboardApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl FretboardApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);

        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("window.close", &["<Ctrl>W"]);

        self.set_accels_for_action("win.empty-chord", &["<Ctrl>E"]);
        self.set_accels_for_action("win.focus-entry", &["<Ctrl>L"]);
        self.set_accels_for_action("win.more-variants", &["<Alt>Return"]);
        self.set_accels_for_action("win.bookmark-chord", &["<Ctrl>D"]);
        self.set_accels_for_action("win.bookmarks", &["<Ctrl><Alt>D"]);
    }

    fn show_about(&self) {
        let about = adw::AboutDialog::from_appdata(
            "/dev/bragefuglseth/Fretboard/metainfo.xml",
            Some("5.0"),
        );

        about.set_developers(&["Brage Fuglseth"]);
        about.set_copyright("© 2023 Brage Fuglseth");
        // Translators: Replace "translator-credits" with your names, one name per line
        about.set_translator_credits(&gettext("translator-credits"));

        about.present(&self.active_window().unwrap());
    }
}
