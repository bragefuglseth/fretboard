use crate::chord_ops::{prettify_chord_name, serialize_chord_name};
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Fretboard/chord-name-entry.ui")]
    pub struct FretboardChordNameEntry {
        #[template_child]
        pub entry: TemplateChild<gtk::Text>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub button: TemplateChild<gtk::Button>,

        pub entry_buffer: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardChordNameEntry {
        const NAME: &'static str = "FretboardChordNameEntry";
        type Type = super::FretboardChordNameEntry;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardChordNameEntry {
        fn constructed(&self) {
            self.parent_constructed();

            let revealer = self.revealer.get();

            self.entry.connect_changed(glib::clone!(@weak self as entry_wrapper => move |entry| {
                let entry_text = entry.text().as_str().to_owned();
                let changed = entry_text != *entry_wrapper.entry_buffer.borrow() && !entry_text.is_empty();
                entry_wrapper.revealer.set_visible(changed);
                entry_wrapper.revealer.set_reveal_child(changed);
            }));

            self.entry.connect_activate(
                glib::clone!(@weak revealer, @weak self as entry => move |_| {
                    revealer.set_visible(false);
                    revealer.set_reveal_child(false);

                    let prettified_name = prettify_chord_name(&entry.entry.text());
                    entry.obj().overwrite_text(&prettified_name);
                    entry.entry.set_position(-1);
                }),
            );

            let entry = self.entry.get();

            self.button
                .connect_clicked(glib::clone!(@weak entry => move |_|{
                    entry.emit_activate();
                }));
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for FretboardChordNameEntry {}
}

glib::wrapper! {
    pub struct FretboardChordNameEntry(ObjectSubclass<imp::FretboardChordNameEntry>)
        @extends gtk::Widget;
}

impl Default for FretboardChordNameEntry {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl FretboardChordNameEntry {
    pub fn entry(&self) -> gtk::Text {
        self.imp().entry.get()
    }

    pub fn serialized_buffer_text(&self) -> String {
        serialize_chord_name(&self.imp().entry_buffer.borrow())
    }

    pub fn overwrite_text(&self, text: &str) {
        let imp = self.imp();
        let text = prettify_chord_name(&text);
        imp.entry_buffer.replace(text.clone());
        imp.entry.set_text(&text);
    }
}
