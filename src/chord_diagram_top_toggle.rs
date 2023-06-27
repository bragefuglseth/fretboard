use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use std::cell::Cell;

#[derive(Default, Clone, Copy, Debug)]
pub enum TopToggleState {
    #[default]
    Off,
    Open,
    Muted,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/bragefuglseth/Fretboard/chord-diagram-top-toggle.ui")]
    pub struct FretboardChordDiagramTopToggle {
        #[template_child]
        pub button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub icon_stack: TemplateChild<gtk::Stack>,

        pub recently_toggled: Cell<bool>,
        pub state: Cell<TopToggleState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardChordDiagramTopToggle {
        const NAME: &'static str = "FretboardChordDiagramTopToggle";
        type Type = super::FretboardChordDiagramTopToggle;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("chord-diagram-top-toggle");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardChordDiagramTopToggle {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_callbacks();
        }
    }

    impl WidgetImpl for FretboardChordDiagramTopToggle {}
}

glib::wrapper! {
    pub struct FretboardChordDiagramTopToggle(ObjectSubclass<imp::FretboardChordDiagramTopToggle>)
        @extends gtk::Widget;
}

impl Default for FretboardChordDiagramTopToggle {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl FretboardChordDiagramTopToggle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn button(&self) -> gtk::ToggleButton {
        self.imp().button.get()
    }

    pub fn set_state(&self, state: TopToggleState) {
        let imp = self.imp();

        imp.button.set_active(match state {
            TopToggleState::Off => false,
            TopToggleState::Open | TopToggleState::Muted => true,
        });

        imp.state.set(state);
        self.update_icon();

        imp.recently_toggled.set(false);
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();

        // A hacky way to get the button state to update properly, better solutions are welcome :)
        imp.button
            .connect_toggled(glib::clone!(@weak imp, @weak self as toggle => move |_| {
                if imp.button.is_active() {
                    imp.state.set(TopToggleState::Open);
                } else {
                    imp.state.set(TopToggleState::Off);
                }

                imp.recently_toggled.set(true);
                toggle.update_icon();
            }));

        imp.button
            .connect_clicked(glib::clone!(@weak imp, @weak self as toggle => move |_| {
                if !imp.recently_toggled.get() {
                    imp.state.set(match imp.state.get() {
                        TopToggleState::Open => TopToggleState::Muted,
                        TopToggleState::Muted => TopToggleState::Open,
                        TopToggleState::Off => TopToggleState::Off,
                    });
                }
                imp.recently_toggled.set(false);
                toggle.update_icon();
            }));
    }

    pub fn update_icon(&self) {
        let imp = self.imp();

        imp.icon_stack
            .set_visible_child_name(match imp.state.get() {
                TopToggleState::Off => "off",
                TopToggleState::Open => "open",
                TopToggleState::Muted => "muted",
            });
    }
}
