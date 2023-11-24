use crate::chord_ops::{enharmonic_equivalent, prettify_chord_name, serialize_chord_name};
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

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
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub button: TemplateChild<gtk::Button>,
        #[template_child]
        pub enharmonic_button: TemplateChild<gtk::Button>,

        pub entry_buffer: RefCell<String>,
        pub has_enharmonic_equivalent: Cell<bool>,
        pub programatically_changed: Cell<bool>,
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
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("enharmonic-clicked").build()]);
            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            let obj = self.obj();

            self.parent_constructed();

            let revealer = self.revealer.get();

            self.entry
                .connect_changed(glib::clone!(@weak self as entry_wrapper => move |entry| {
                    if entry_wrapper.programatically_changed.get() {
                        entry_wrapper.programatically_changed.set(false);
                        return;
                    }

                    let entry_text = entry.text().as_str().to_owned();
                    let changed = entry_text != *entry_wrapper.entry_buffer.borrow();
                    let empty = entry_text.is_empty();

                    if changed && !empty {
                        entry_wrapper.stack.set_visible_child_name("confirm-button");
                        entry_wrapper.revealer.set_visible(true);
                        entry_wrapper.revealer.set_reveal_child(true);
                    } else if entry_wrapper.has_enharmonic_equivalent.get() && !empty {
                        entry_wrapper.revealer.set_visible(true);
                        entry_wrapper.revealer.set_reveal_child(true);
                    } else {
                        entry_wrapper.revealer.set_visible(false);
                        entry_wrapper.revealer.set_reveal_child(false);
                    }
                }));

            self.entry.connect_activate(
                glib::clone!(@weak revealer, @weak self as entry_wrapper => move |_| {
                    revealer.set_visible(false);
                    revealer.set_reveal_child(false);

                    let prettified_name = prettify_chord_name(&entry_wrapper.entry.text());
                    entry_wrapper.obj().overwrite_text(&prettified_name);
                    entry_wrapper.entry.set_position(-1);
                }),
            );

            let entry = self.entry.get();

            self.button
                .connect_clicked(glib::clone!(@weak entry => move |_|{
                    entry.emit_activate();
                }));

            self.enharmonic_button.connect_clicked(glib::clone!(@weak obj, @weak entry, @weak self as entry_wrapper => move |btn| {
                let enharmonic = btn.label().map(|gs| gs.to_string()).unwrap_or(String::from(""));
                let modified_name = format!("{}{}", enharmonic, entry.text().chars().skip(2).collect::<String>());
                obj.imp().programatically_changed.set(true);
                obj.overwrite_text(&modified_name);
                entry_wrapper.obj().emit_by_name::<()>("enharmonic-clicked", &[]);
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
        self.calculate_enharmonic_equivalent(&text);
    }

    pub fn calculate_enharmonic_equivalent(&self, chord_name: &str) {
        let imp = self.imp();

        if let Some(equivalent) = enharmonic_equivalent(&serialize_chord_name(chord_name)) {
            imp.enharmonic_button
                .set_label(&prettify_chord_name(equivalent));
            imp.revealer.set_visible(true);
            imp.revealer.set_reveal_child(true);
            imp.stack.set_visible_child_name("enharmonic-equivalent");

            imp.has_enharmonic_equivalent.set(true);
        } else {
            imp.has_enharmonic_equivalent.set(false);
        }
    }
}
