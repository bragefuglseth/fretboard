use crate::chord_diagram_toggle::FretboardChordDiagramToggle;
use crate::chord_diagram_top_toggle::{FretboardChordDiagramTopToggle, TopToggleState};
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};

const STRINGS: usize = 6;
const FRETS: usize = 5;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Fretboard/chord-diagram.ui")]
    pub struct FretboardChordDiagram {
        #[template_child]
        top_row: TemplateChild<gtk::Box>,
        #[template_child]
        diagram_backdrop: TemplateChild<gtk::Picture>,
        #[template_child]
        grid: TemplateChild<gtk::Grid>,

        pub chord: RefCell<Vec<Option<usize>>>,

        pub top_toggles: RefCell<Vec<FretboardChordDiagramTopToggle>>,
        pub toggles: RefCell<Vec<Vec<gtk::ToggleButton>>>,
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

            // Setup top toggles
            for _ in 0..STRINGS {
                let top_toggle = FretboardChordDiagramTopToggle::new();
                self.top_row.append(&top_toggle);
                self.top_toggles.borrow_mut().push(top_toggle);
            }

            // Setup toggles
            for string_num in 0..STRINGS {
                let mut current_string_toggles = Vec::new();

                for fret_num in 0..FRETS {
                    let toggle = FretboardChordDiagramToggle::new();
                    toggle.button().set_group(Some(
                        &self.top_toggles.borrow().get(string_num).unwrap().button(),
                    ));

                    self.grid
                        .attach(&toggle, string_num as i32, fret_num as i32, 1, 1);

                    current_string_toggles.push(toggle.button());
                }

                self.toggles.borrow_mut().push(current_string_toggles);
            }

            self.chord.replace(vec![None, Some(3), Some(2), Some(0), Some(1), Some(0)]);

            self.obj().update_toggles();
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

impl FretboardChordDiagram {
    fn update_toggles(&self) {
        let chord = self.imp().chord.borrow();
        let top_toggles = self.imp().top_toggles.borrow();
        let toggles = self.imp().toggles.borrow();

        for string in 0..STRINGS {
            let top_toggle = top_toggles.get(string).unwrap();

            match chord.get(string).expect("chords vec has len of 6") {
                None => top_toggle.set_state(TopToggleState::Muted),
                Some(0) => top_toggle.set_state(TopToggleState::Open),
                Some(n) if *n < FRETS => {
                    toggles.get(string).unwrap().get(*n - 1).unwrap().set_active(true);
                }
                Some(_) => top_toggle.set_state(TopToggleState::Muted),
            }
        }
    }
}
