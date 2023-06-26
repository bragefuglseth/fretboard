use crate::chord_diagram_toggle::FretboardChordDiagramToggle;
use crate::chord_diagram_top_toggle::{FretboardChordDiagramTopToggle, TopToggleState};
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Fretboard/chord-diagram.ui")]
    pub struct FretboardChordDiagram {
        #[template_child]
        diagram_backdrop: TemplateChild<gtk::Picture>,
        #[template_child]
        string_1_top: TemplateChild<FretboardChordDiagramTopToggle>,
        #[template_child]
        string_2_top: TemplateChild<FretboardChordDiagramTopToggle>,
        #[template_child]
        string_3_top: TemplateChild<FretboardChordDiagramTopToggle>,
        #[template_child]
        string_4_top: TemplateChild<FretboardChordDiagramTopToggle>,
        #[template_child]
        string_5_top: TemplateChild<FretboardChordDiagramTopToggle>,
        #[template_child]
        string_6_top: TemplateChild<FretboardChordDiagramTopToggle>,
        #[template_child]
        string_1_fret_1: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_1_fret_2: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_1_fret_3: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_1_fret_4: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_1_fret_5: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_2_fret_1: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_2_fret_2: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_2_fret_3: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_2_fret_4: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_2_fret_5: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_3_fret_1: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_3_fret_2: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_3_fret_3: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_3_fret_4: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_3_fret_5: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_4_fret_1: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_4_fret_2: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_4_fret_3: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_4_fret_4: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_4_fret_5: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_5_fret_1: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_5_fret_2: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_5_fret_3: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_5_fret_4: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_5_fret_5: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_6_fret_1: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_6_fret_2: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_6_fret_3: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_6_fret_4: TemplateChild<FretboardChordDiagramToggle>,
        #[template_child]
        string_6_fret_5: TemplateChild<FretboardChordDiagramToggle>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardChordDiagram {
        const NAME: &'static str = "FretboardChordDiagram";
        type Type = super::FretboardChordDiagram;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("chord-diagram");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardChordDiagram {
        fn constructed(&self) {
            self.parent_constructed();

            self.diagram_backdrop.set_resource(Some(
                "/dev/bragefuglseth/Fretboard/chord-diagram-backdrop.svg",
            ));

            let group_1 = self.string_1_top.button();

            self.string_1_fret_1.button().set_group(Some(&group_1));
            self.string_1_fret_2.button().set_group(Some(&group_1));
            self.string_1_fret_3.button().set_group(Some(&group_1));
            self.string_1_fret_4.button().set_group(Some(&group_1));
            self.string_1_fret_5.button().set_group(Some(&group_1));

            let group_2 = self.string_2_top.button();

            self.string_2_fret_1.button().set_group(Some(&group_2));
            self.string_2_fret_2.button().set_group(Some(&group_2));
            self.string_2_fret_3.button().set_group(Some(&group_2));
            self.string_2_fret_4.button().set_group(Some(&group_2));
            self.string_2_fret_5.button().set_group(Some(&group_2));

            let group_3 = self.string_3_top.button();

            self.string_3_fret_1.button().set_group(Some(&group_3));
            self.string_3_fret_2.button().set_group(Some(&group_3));
            self.string_3_fret_3.button().set_group(Some(&group_3));
            self.string_3_fret_4.button().set_group(Some(&group_3));
            self.string_3_fret_5.button().set_group(Some(&group_3));

            let group_4 = self.string_4_top.button();

            self.string_4_fret_1.button().set_group(Some(&group_4));
            self.string_4_fret_2.button().set_group(Some(&group_4));
            self.string_4_fret_3.button().set_group(Some(&group_4));
            self.string_4_fret_4.button().set_group(Some(&group_4));
            self.string_4_fret_5.button().set_group(Some(&group_4));

            let group_5 = self.string_5_top.button();

            self.string_5_fret_1.button().set_group(Some(&group_5));
            self.string_5_fret_2.button().set_group(Some(&group_5));
            self.string_5_fret_3.button().set_group(Some(&group_5));
            self.string_5_fret_4.button().set_group(Some(&group_5));
            self.string_5_fret_5.button().set_group(Some(&group_5));

            let group_6 = self.string_6_top.button();

            self.string_6_fret_1.button().set_group(Some(&group_6));
            self.string_6_fret_2.button().set_group(Some(&group_6));
            self.string_6_fret_3.button().set_group(Some(&group_6));
            self.string_6_fret_4.button().set_group(Some(&group_6));
            self.string_6_fret_5.button().set_group(Some(&group_6));

            self.string_1_top.set_state(TopToggleState::Open);
            self.string_2_top.set_state(TopToggleState::Open);
            self.string_3_top.set_state(TopToggleState::Open);
            self.string_4_top.set_state(TopToggleState::Open);
            self.string_5_top.set_state(TopToggleState::Open);
            self.string_6_top.set_state(TopToggleState::Open);
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for FretboardChordDiagram {}
}

glib::wrapper! {
    pub struct FretboardChordDiagram(ObjectSubclass<imp::FretboardChordDiagram>)
        @extends gtk::Widget;
}

impl Default for FretboardChordDiagram {
    fn default() -> Self {
        glib::Object::new()
    }
}
