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
        pub entry: TemplateChild<gtk::Entry>,

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
            klass.set_css_name("chord-name-entry");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardChordNameEntry {
        fn constructed(&self) {
            self.parent_constructed();

            self.entry.connect_changed(glib::clone!(@weak self as entry_wrapper => move |entry| {
                if &entry.text().as_str() != &entry_wrapper.entry_buffer.borrow().clone() {
                    entry.set_secondary_icon_name(Some("checkmark-large-symbolic"));
                } else {
                    entry.set_secondary_icon_name(None);
                }
            }));

            self.entry.connect_activate(|entry| {
                entry.set_secondary_icon_name(None);
            });

            self.entry.connect_icon_release(|entry, _| {
                entry.emit_by_name::<()>("activate", &[]);
            });
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
    pub fn entry(&self) -> gtk::Entry {
        self.imp().entry.get()
    }
}
