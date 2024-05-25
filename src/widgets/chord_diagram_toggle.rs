use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/widgets/chord_diagram_toggle.blp")]
    pub struct FretboardChordDiagramToggle {
        #[template_child]
        pub button: TemplateChild<gtk::ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardChordDiagramToggle {
        const NAME: &'static str = "FretboardChordDiagramToggle";
        type Type = super::FretboardChordDiagramToggle;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("chord-diagram-toggle");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardChordDiagramToggle {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for FretboardChordDiagramToggle {}
}

glib::wrapper! {
    pub struct FretboardChordDiagramToggle(ObjectSubclass<imp::FretboardChordDiagramToggle>)
        @extends gtk::Widget;
}

impl Default for FretboardChordDiagramToggle {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl FretboardChordDiagramToggle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn button(&self) -> gtk::ToggleButton {
        self.imp().button.get()
    }
}
