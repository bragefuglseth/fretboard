use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;
use i18n_format::i18n_fmt;
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
    #[template(file = "src/widgets/chord_diagram_top_toggle.blp")]
    pub struct FretboardChordDiagramTopToggle {
        #[template_child]
        pub button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub icon_stack: TemplateChild<gtk::Stack>,

        // these two booleans are used to avoid side effects when changing button state.
        // the system is pretty bad architecturally, but it works, and the madness is contained
        // within this module, so I don't see much reason for changing it.
        pub programmatically_toggled: Cell<bool>,
        pub recently_toggled: Cell<bool>,

        pub state: Cell<TopToggleState>,
        pub number: Cell<usize>,
        pub note_name: Cell<&'static str>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardChordDiagramTopToggle {
        const NAME: &'static str = "FretboardChordDiagramTopToggle";
        type Type = super::FretboardChordDiagramTopToggle;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.set_layout_manager_type::<gtk::BinLayout>();
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
            obj.add_css_class("fretboard-chord-diagram-top-toggle");
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
    pub fn new(number: usize, note_name: &'static str) -> Self {
        let toggle = Self::default();
        toggle.imp().number.set(number);
        toggle.imp().note_name.set(note_name);
        toggle
    }

    pub fn button(&self) -> gtk::ToggleButton {
        self.imp().button.get()
    }

    pub fn set_state(&self, state: TopToggleState) {
        let imp = self.imp();

        imp.programmatically_toggled.set(true);
        imp.button.set_active(match state {
            TopToggleState::Off => false,
            TopToggleState::Open | TopToggleState::Muted => true,
        });
        imp.programmatically_toggled.set(false);

        imp.state.set(state);
        self.update_icon();

        imp.recently_toggled.set(false);
    }

    pub fn state(&self) -> TopToggleState {
        self.imp().state.get()
    }

    pub fn set_note_name(&self, note_name: &'static str) {
        self.imp().note_name.set(note_name);
        self.update_tooltip();
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();

        // A hacky way to get the button state to update properly, better solutions are welcome :)
        imp.button
            .connect_toggled(glib::clone!(@weak imp, @weak self as toggle => move |_| {
                if !imp.programmatically_toggled.get() {
                    if imp.button.is_active() {
                        imp.state.set(TopToggleState::Open);
                    } else {
                        imp.state.set(TopToggleState::Off);
                    }

                    imp.recently_toggled.set(true);
                    toggle.update_icon();
                }
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

    fn update_tooltip(&self) {
        let imp = self.imp();

        let tooltip_text = match imp.state.get() {
            TopToggleState::Off => i18n_fmt!(
                i18n_fmt("Not Open ({})", self.imp().note_name.get())
            ),
            TopToggleState::Muted => i18n_fmt!(
                // translators: The text between the `{}` markers is the note of the muted string.
                i18n_fmt("Muted ({})", self.imp().note_name.get())
            ),
            TopToggleState::Open => i18n_fmt!(
                // translators: The text between the `{}` markers is the note of the open string. "Open" is an adjective, not a verb.
                i18n_fmt("Open ({})", self.imp().note_name.get())
            ),
        };

        imp.button.set_tooltip_text(Some(&tooltip_text));
    }

    pub fn update_icon(&self) {
        let imp = self.imp();

        imp.icon_stack
            .set_visible_child_name(match imp.state.get() {
                TopToggleState::Off => "off",
                TopToggleState::Open => "open",
                TopToggleState::Muted => "muted",
            });

        let n = imp.number.get();

        let a11y_label = match imp.state.get() {
            // translators: {} in the following strings will be replaced by a number ("String 1, Not Open")
            TopToggleState::Off => gettext!("String {}, Not Open", n),
            TopToggleState::Muted => gettext!("String {}, Muted", n),
            // translators: "open" is an adjective, not a verb.
            TopToggleState::Open => gettext!("String {}, Open", n),
        };

        imp.button
            .update_property(&[gtk::accessible::Property::Label(&a11y_label)]);

        self.update_tooltip();
    }
}
