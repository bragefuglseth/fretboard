use crate::barre_spin::FretboardBarreSpin;
use crate::chord_diagram_toggle::FretboardChordDiagramToggle;
use crate::chord_diagram_top_toggle::{FretboardChordDiagramTopToggle, TopToggleState};
use crate::chord_ops::*;
use crate::window::GuitarType;
use adw::subclass::prelude::*;
use glib::{closure_local, subclass::Signal};
use gtk::glib;
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

const STRINGS: usize = 6;
const NOTE_OFFSETS: [usize; STRINGS] = [7, 0, 5, 10, 2, 7];
const FRETS: usize = 5;
const SCALE_SIZE: usize = 12;

pub enum SpinMessage {
    Increment,
    Decrement,
}

mod imp {
    use super::*;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties( wrapper_type = super::FretboardChordDiagram )]
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
        pub guitar_type: Cell<GuitarType>,

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

            // The direction of the diagram is strictly visual
            self.top_row.set_direction(gtk::TextDirection::Ltr);
            self.grid.set_direction(gtk::TextDirection::Ltr);

            let style_manager = adw::StyleManager::default();

            style_manager.connect_dark_notify(glib::clone!(@weak obj => move |_| {
                obj.update_style();
            }));

            style_manager.connect_high_contrast_notify(glib::clone!(@weak obj => move |_| {
                obj.update_style();
            }));

            obj.update_style();

            self.diagram_backdrop
                .set_accessible_role(gtk::AccessibleRole::Presentation);

            // Setup top toggles
            for i in 0..STRINGS {
                let top_toggle =
                    FretboardChordDiagramTopToggle::new(STRINGS - i, note_name(NOTE_OFFSETS[i]));
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
                let mut current_string_toggles = Vec::with_capacity(FRETS);

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

            barre_spin.connect_closure(
                "user-changed-value",
                false,
                closure_local!(@strong obj => move |_spin: FretboardBarreSpin, string: &str| {
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

        self.set_neck_position(lowest_fingered_fret(chord).unwrap_or(1));

        self.imp().barre_spin.set_value(self.neck_position());

        self.update_visuals();
        self.update_barre_visuals();
    }

    fn update_chord(&self) {
        let imp = self.imp();

        let top_toggles = imp.top_toggles.borrow();
        let toggles = imp.toggles.borrow();

        let mut chord: [Option<usize>; 6] = [None; 6];

        // accomodate left/right-handedness
        let string_range: Vec<usize> = match imp.guitar_type.get() {
            GuitarType::RightHanded => (0..STRINGS).collect(),
            GuitarType::LeftHanded => (0..STRINGS).rev().collect(),
        };

        for i in 0..STRINGS {
            let string = *string_range.get(i).unwrap();

            let top_toggle = top_toggles.get(string).unwrap();

            if matches!(top_toggle.state(), TopToggleState::Muted) {
                *chord.get_mut(i).unwrap() = None;
            } else if matches!(top_toggle.state(), TopToggleState::Open) {
                *chord.get_mut(i).unwrap() = Some(0);
            } else {
                let pos = toggles
                    .get(string)
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
        let imp = self.imp();

        let chord = imp.chord.get();
        // Adjust chord so it's positioned relatively to the neck position
        let adjusted_chord = adjust_chord(chord, self.neck_position());

        let top_toggles = imp.top_toggles.borrow();
        let toggles = imp.toggles.borrow();

        let string_range: Vec<usize> = match imp.guitar_type.get() {
            GuitarType::RightHanded => (0..STRINGS).collect(),
            GuitarType::LeftHanded => (0..STRINGS).rev().collect(),
        };

        for i in 0..STRINGS {
            let string = *string_range.get(i).unwrap();

            let top_toggle = top_toggles.get(string).unwrap();

            match adjusted_chord.get(i).expect("chord has len of 6") {
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

            let offset = NOTE_OFFSETS.get(i).unwrap();
            top_toggle.set_note_name(note_name(*offset));

            for (num, toggle) in toggles.get(string).unwrap().iter().enumerate() {
                toggle.set_tooltip_text(Some(note_name(
                    offset + num + self.neck_position() as usize,
                )));
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

        // in high contrast mode, just use the dark mode assets for light mode and vice versa
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

    pub fn set_guitar_type(&self, guitar_type: GuitarType) {
        let imp = self.imp();
        imp.guitar_type.replace(guitar_type);
        self.update_visuals();

        let barre_alignment = match guitar_type {
            GuitarType::RightHanded => gtk::Align::End,
            GuitarType::LeftHanded => gtk::Align::Start,
        };

        for barre_picture in [
            imp.barre_2_image.get(),
            imp.barre_3_image.get(),
            imp.barre_4_image.get(),
            imp.barre_5_image.get(),
            imp.barre_6_image.get(),
        ] {
            barre_picture.set_halign(barre_alignment);
        }
    }
}

fn note_name(input: usize) -> &'static str {
    match input % SCALE_SIZE {
        0 => "A",
        1 => "A♯",
        2 => "B",
        3 => "C",
        4 => "C♯",
        5 => "D",
        6 => "D♯",
        7 => "E",
        8 => "F",
        9 => "F♯",
        10 => "G",
        11 => "G♯",
        _ => unreachable!("root note above 11"),
    }
}
