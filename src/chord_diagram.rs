use crate::barre_spin::FretboardBarreSpin;
use crate::chord_diagram_toggle::FretboardChordDiagramToggle;
use crate::chord_diagram_top_toggle::{FretboardChordDiagramTopToggle, TopToggleState};
use adw::subclass::prelude::*;
use glib::{closure_local, subclass::Signal};
use gtk::glib;
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

const STRINGS: usize = 6;
const FRETS: usize = 4;

// These are always shown in fret position 1
const SPECIAL_CASE_CHORDS: [[Option<usize>; 6]; 17] = [
    [None, Some(0), Some(2), Some(2), Some(2), Some(0)], // A
    [None, Some(0), Some(2), Some(2), None, Some(0)],    // A5
    [None, Some(0), Some(2), Some(0), Some(2), Some(0)], // A7
    [None, Some(0), Some(2), Some(2), Some(3), Some(0)], // Asus4
    [None, Some(2), Some(0), Some(2), Some(0), Some(2)], // Bm7
    [None, Some(3), Some(2), Some(0), Some(0), Some(0)], // Cmaj7
    [None, None, Some(0), Some(2), Some(3), Some(2)],    // D
    [None, None, Some(0), Some(2), Some(3), None],       // D5
    [None, None, Some(0), Some(2), Some(0), Some(2)],    // D6
    [None, None, Some(0), Some(2), Some(2), Some(2)],    // Dmaj7
    [None, None, Some(0), Some(2), Some(3), Some(3)],    // Dsus4
    [Some(0), Some(2), Some(2), Some(0), Some(0), Some(0)], // Em
    [Some(0), Some(2), Some(2), Some(0), Some(2), Some(0)], // Em6
    [Some(0), Some(2), Some(0), Some(0), Some(0), Some(0)], // Em7
    [Some(0), Some(2), Some(2), Some(2), Some(0), Some(0)], // Esus4
    [Some(3), Some(2), Some(0), Some(0), Some(0), Some(3)], // G
    [Some(3), Some(2), Some(0), Some(0), Some(0), Some(0)], // G6
];

pub enum SpinMessage {
    Increment,
    Decrement,
}

mod imp {
    use super::*;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties( wrapper = super::FretboardChordDiagram )]
    #[template(resource = "/dev/bragefuglseth/Fretboard/chord-diagram.ui")]
    pub struct FretboardChordDiagram {
        #[template_child]
        pub top_row: TemplateChild<gtk::Box>,
        #[template_child]
        pub diagram_backdrop: TemplateChild<gtk::Picture>,
        #[template_child]
        pub barre_overlay_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub barre_2_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub barre_3_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub barre_4_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub barre_5_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub barre_6_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub grid: TemplateChild<gtk::Grid>,
        #[template_child]
        pub barre_spin: TemplateChild<FretboardBarreSpin>,

        pub chord: Cell<[Option<usize>; 6]>,

        #[property(get, set)]
        pub neck_position: Cell<u8>,

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
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> =
                Lazy::new(|| vec![Signal::builder("user-changed-chord").build()]);
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let style_manager = adw::StyleManager::default();

            style_manager.connect_dark_notify(glib::clone!(@weak obj => move |_| {
                obj.update_style();
            }));

            style_manager.connect_high_contrast_notify(glib::clone!(@weak obj => move |_| {
                obj.update_style();
            }));

            obj.update_style();

            // Setup top toggles
            for _ in 0..STRINGS {
                let top_toggle = FretboardChordDiagramTopToggle::new();
                top_toggle
                    .button()
                    .connect_clicked(glib::clone!(@weak obj => move |_| {
                        obj.update_chord();
                        obj.emit_by_name::<()>("user-changed-chord", &[]);
                    }));
                self.top_row.append(&top_toggle);
                self.top_toggles.borrow_mut().push(top_toggle);
            }

            // Setup toggles
            for string_num in 0..STRINGS {
                let mut current_string_toggles = Vec::new();

                for fret_num in 0..FRETS {
                    let toggle = FretboardChordDiagramToggle::new();
                    toggle
                        .button()
                        .connect_clicked(glib::clone!(@weak obj => move |_| {
                            obj.update_chord();
                            obj.emit_by_name::<()>("user-changed-chord", &[]);
                        }));
                    toggle.button().set_group(Some(
                        &self.top_toggles.borrow().get(string_num).unwrap().button(),
                    ));

                    self.grid
                        .attach(&toggle, string_num as i32, fret_num as i32, 1, 1);

                    current_string_toggles.push(toggle.button());
                }

                self.toggles.borrow_mut().push(current_string_toggles);
            }

            let barre_spin = self.barre_spin.get();

            let obj = obj.clone();

            barre_spin.connect_closure(
                "user-changed-value",
                false,
                closure_local!(move |_spin: FretboardBarreSpin, string: &str| {
                    let message = match string {
                        "increment" => SpinMessage::Increment,
                        "decrement" => SpinMessage::Decrement,
                        _ => panic!("unknown message from spin button"),
                    };
                    obj.update_neck_position(message);
                    obj.emit_by_name::<()>("user-changed-chord", &[]);
                }),
            );

            self.obj().update_visuals();
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
    pub fn set_chord(&self, chord: [Option<usize>; 6]) {
        self.imp().chord.set(chord);

        self.set_neck_position(
            if SPECIAL_CASE_CHORDS
                .iter()
                .any(|&special_chord| special_chord == chord)
            {
                1
            } else {
                find_lowest_non_zero_fret(chord).unwrap_or(1)
            },
        );

        self.imp().barre_spin.set_value(self.neck_position());

        self.update_visuals();
        self.update_barre_visuals();
    }

    fn update_chord(&self) {
        let top_toggles = self.imp().top_toggles.borrow();
        let toggles = self.imp().toggles.borrow();

        let mut chord: [Option<usize>; 6] = [None; 6];

        for i in 0..STRINGS {
            let top_toggle = top_toggles.get(i).unwrap();

            if matches!(top_toggle.state(), TopToggleState::Muted) {
                *chord.get_mut(i).unwrap() = None;
            } else if matches!(top_toggle.state(), TopToggleState::Open) {
                *chord.get_mut(i).unwrap() = Some(0);
            } else {
                let pos = toggles
                    .get(i)
                    .unwrap()
                    .iter()
                    .position(|toggle| toggle.is_active())
                    .unwrap()
                    + self.imp().neck_position.get() as usize;

                *chord.get_mut(i).unwrap() = Some(pos);
            }
        }

        self.imp().chord.set(chord);
        self.update_barre_visuals();
    }

    pub fn update_neck_position(&self, message: SpinMessage) {
        let chord = self.imp().chord.get();

        let change = match message {
            SpinMessage::Increment => 1,
            SpinMessage::Decrement => -1,
        };

        if chord.iter().all(|&value| matches!(value, None | Some(0))) {
            let new_pos = (self.neck_position() as i32 + change) as u8;
            self.set_neck_position(new_pos);
            return;
        }

        let new_chord: [Option<usize>; 6] = chord
            .iter()
            .map(|&val| {
                val.map(|note| {
                    if note == 0 {
                        0
                    } else {
                        (note as i32 + change) as usize
                    }
                })
            })
            .collect::<Vec<Option<usize>>>()
            .try_into()
            .unwrap();

        self.set_chord(new_chord);
    }

    fn update_visuals(&self) {
        let chord = self.imp().chord.get();
        // Adjust chord so it's positioned relatively to the neck position
        let adjusted_chord = adjust_chord(chord, self.neck_position());

        let top_toggles = self.imp().top_toggles.borrow();
        let toggles = self.imp().toggles.borrow();

        for string in 0..STRINGS {
            let top_toggle = top_toggles.get(string).unwrap();

            match adjusted_chord.get(string).expect("chord has len of 6") {
                None => top_toggle.set_state(TopToggleState::Muted),
                Some(0) => top_toggle.set_state(TopToggleState::Open),
                Some(n) if *n <= FRETS => {
                    toggles
                        .get(string)
                        .unwrap()
                        .get(*n - 1)
                        .unwrap()
                        .set_active(true);
                }
                Some(_) => top_toggle.set_state(TopToggleState::Muted),
            }
        }
    }

    fn update_barre_visuals(&self) {
        let chord = self.imp().chord.get();

        let chord = adjust_chord(chord, self.neck_position());

        let barre_length = find_barre_length(chord);

        let barre_stack = self.imp().barre_overlay_stack.get();
        barre_stack.set_visible_child_name(match barre_length {
            2 => "barre-2",
            3 => "barre-3",
            4 => "barre-4",
            5 => "barre-5",
            6 => "barre-6",
            _ => "empty",
        });
    }

    fn update_style(&self) {
        let app_style = adw::StyleManager::default();

        // in hight contrast mode, just use the dark mode assets for light mode and vice versa
        let suffix = match (app_style.is_dark(), app_style.is_high_contrast()) {
            (false, false) | (true, true) => "light",
            (true, false) | (false, true) => "dark",
        };

        let backdrop = self.imp().diagram_backdrop.get();
        backdrop.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/chord-diagram-backdrop-{suffix}.svg"
        )));

        let barre_2 = self.imp().barre_2_image.get();
        let barre_3 = self.imp().barre_3_image.get();
        let barre_4 = self.imp().barre_4_image.get();
        let barre_5 = self.imp().barre_5_image.get();
        let barre_6 = self.imp().barre_6_image.get();

        barre_2.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/barre-2-{suffix}.svg"
        )));
        barre_3.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/barre-3-{suffix}.svg"
        )));
        barre_4.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/barre-4-{suffix}.svg"
        )));
        barre_5.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/barre-5-{suffix}.svg"
        )));
        barre_6.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/barre-6-{suffix}.svg"
        )));
    }
}

// ChordOPS 😎️

// find barre length of *adjusted* chords (lowest fingered fret is positioned @ 1)
fn find_barre_length(chord: [Option<usize>; 6]) -> usize {
    if chord
        .iter()
        .filter(|&&option| option == Some(1_usize))
        .count()
        .lt(&2)
    {
        return 0;
    }

    let mut barre_length = 0;

    let chord_reversed = chord.iter().rev().enumerate();

    let mut chord_reversed_next = chord.iter().rev();
    chord_reversed_next.next();

    let mut note_count = 0;

    for (num, val) in chord_reversed {
        if val == &Some(1_usize) {
            barre_length = num + 1;
            note_count += 1;
        }

        let next = chord_reversed_next.next();
        if next == Some(&Some(0_usize))
            || next == Some(&None)
            || val == &Some(0_usize)
            || val.is_none()
        {
            break;
        }
    }
    if note_count > 1 {
        barre_length
    } else {
        0
    }
}

fn find_lowest_non_zero_fret(chord: [Option<usize>; 6]) -> Option<u8> {
    chord
        .iter()
        .filter_map(|&option| option)
        .filter(|&val| val > 0)
        .min()
        .map(|val| val as u8)
}

fn adjust_chord(chord: [Option<usize>; 6], barre: u8) -> [Option<usize>; 6] {
    chord
        .iter()
        .map(|option| {
            option.map(|value| {
                if value == 0 {
                    0
                } else {
                    value - (if barre == 0 { barre } else { barre - 1 }) as usize
                }
            })
        })
        .collect::<Vec<Option<usize>>>()
        .try_into()
        .unwrap()
}
