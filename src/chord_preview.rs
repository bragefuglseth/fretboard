use crate::chord_ops::*;
use crate::window::GuitarType;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};

const STRINGS: usize = 6;
const FRETS: usize = 5;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Fretboard/chord-preview.ui")]
    pub struct FretboardChordPreview {
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
        pub dots_grid: TemplateChild<gtk::Grid>,
        #[template_child]
        pub neck_position_label: TemplateChild<gtk::Label>,

        pub top_symbols: RefCell<Vec<gtk::Image>>,
        pub dots: RefCell<Vec<Vec<gtk::Box>>>,

        pub chord_name: RefCell<String>,
        pub chord: Cell<[Option<usize>; 6]>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardChordPreview {
        const NAME: &'static str = "FretboardChordPreview";
        type Type = super::FretboardChordPreview;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardChordPreview {
        fn constructed(&self) {
            self.parent_constructed();

            // The direction of the diagram is strictly visual
            self.top_row.set_direction(gtk::TextDirection::Ltr);
            self.dots_grid.set_direction(gtk::TextDirection::Ltr);

            for i in 0..STRINGS {
                let mut dot_column = Vec::new();

                let top_symbol = gtk::Image::builder()
                    .valign(gtk::Align::Center)
                    .halign(gtk::Align::Center)
                    .hexpand(true)
                    .css_classes(["fretboard-chord-diagram-top-toggle"])
                    .build();

                self.top_row.append(&top_symbol);

                self.top_symbols.borrow_mut().push(top_symbol);

                for fret_num in 0..FRETS {
                    let dot = gtk::Box::builder()
                        .width_request(20)
                        .height_request(20)
                        .valign(gtk::Align::Center)
                        .halign(gtk::Align::Center)
                        .vexpand(true)
                        .hexpand(true)
                        .css_classes(["fretboard-chord-preview-dot"])
                        .build();

                    self.dots_grid.attach(&dot, i as i32, fret_num as i32, 1, 1);

                    dot_column.push(dot);
                }

                self.dots.borrow_mut().push(dot_column);
            }

            let obj = self.obj();

            let style_manager = adw::StyleManager::default();

            style_manager.connect_dark_notify(glib::clone!(@weak obj => move |_| {
                obj.update_style();
            }));

            style_manager.connect_high_contrast_notify(glib::clone!(@weak obj => move |_| {
                obj.update_style();
            }));

            obj.update_style();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for FretboardChordPreview {}
}

glib::wrapper! {
    pub struct FretboardChordPreview(ObjectSubclass<imp::FretboardChordPreview>)
        @extends gtk::Widget;
}

impl Default for FretboardChordPreview {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl FretboardChordPreview {
    pub fn with_chord(chord: [Option<usize>; 6], guitar_type: GuitarType) -> Self {
        let preview = Self::default();
        preview.set_chord(chord, guitar_type);
        preview
    }

    pub fn set_chord(&self, chord: [Option<usize>; 6], guitar_type: GuitarType) {
        let imp = self.imp();
        imp.chord.set(chord);

        self.reset_visuals();

        let neck_position = find_lowest_non_zero_fret(chord).unwrap_or(1);

        let adjusted_chord = adjust_chord(chord, neck_position);

        let string_range = match guitar_type {
            GuitarType::RightHanded => (0..STRINGS).collect::<Vec<_>>().into_iter(),
            GuitarType::LeftHanded => (0..STRINGS).rev().collect::<Vec<_>>().into_iter(),
        };

        for (value, n) in adjusted_chord.iter().zip(string_range) {
            match value {
                None => imp
                    .top_symbols
                    .borrow()
                    .get(n)
                    .unwrap()
                    .set_icon_name(Some("cross-large-symbolic")),
                Some(0) => {
                    let top_symbols = imp.top_symbols.borrow();
                    let top_symbol = top_symbols.get(n).unwrap();
                    top_symbol.set_icon_name(Some("circle-outline-thick-symbolic"));
                    top_symbol.add_css_class("open");
                }
                Some(fret) => imp
                    .dots
                    .borrow()
                    .get(n)
                    .unwrap()
                    .get(*fret - 1)
                    .unwrap()
                    .add_css_class("active"),
            }

            imp.neck_position_label
                .set_label(&neck_position.to_string());
        }

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

        let barre_length = find_barre_length(adjusted_chord);

        let barre_stack = imp.barre_overlay_stack.get();
        barre_stack.set_visible_child_name(match barre_length {
            2 => "barre-2",
            3 => "barre-3",
            4 => "barre-4",
            5 => "barre-5",
            6 => "barre-6",
            _ => "empty",
        });
    }

    fn reset_visuals(&self) {
        let imp = self.imp();

        for top_symbol in imp.top_symbols.borrow().iter() {
            top_symbol.remove_css_class("open");
            top_symbol.set_icon_name(Some("dot-symbolic"));
        }

        for dot in imp.dots.borrow().iter().flatten() {
            dot.remove_css_class("active");
        }

        imp.neck_position_label.set_label("1");
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
            "/dev/bragefuglseth/Fretboard/chord-preview-backdrop-{suffix}.svg"
        )));

        let barre_2 = self.imp().barre_2_image.get();
        let barre_3 = self.imp().barre_3_image.get();
        let barre_4 = self.imp().barre_4_image.get();
        let barre_5 = self.imp().barre_5_image.get();
        let barre_6 = self.imp().barre_6_image.get();

        barre_2.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/prv-barre-2-{suffix}.svg"
        )));
        barre_3.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/prv-barre-3-{suffix}.svg"
        )));
        barre_4.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/prv-barre-4-{suffix}.svg"
        )));
        barre_5.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/prv-barre-5-{suffix}.svg"
        )));
        barre_6.set_resource(Some(&format!(
            "/dev/bragefuglseth/Fretboard/prv-barre-6-{suffix}.svg"
        )));
    }
}
