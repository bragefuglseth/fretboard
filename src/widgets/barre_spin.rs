use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;
use gtk::prelude::*;
use once_cell::sync::Lazy;
use std::cell::Cell;

const MIN_VALUE: u8 = 1;
const MAX_VALUE: u8 = 12;

mod imp {
    use super::*;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::FretboardBarreSpin)]
    #[template(file = "src/widgets/barre_spin.blp")]
    pub struct FretboardBarreSpin {
        #[template_child]
        pub increment_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub decrement_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        #[property(get, set, minimum = MIN_VALUE, maximum = MAX_VALUE, default = MIN_VALUE)]
        pub value: Cell<u8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FretboardBarreSpin {
        const NAME: &'static str = "FretboardBarreSpin";
        type Type = super::FretboardBarreSpin;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.set_accessible_role(gtk::AccessibleRole::SpinButton);
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("barre-spin");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FretboardBarreSpin {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![Signal::builder("user-changed-value")
                    .param_types([str::static_type()])
                    .build()]
            });
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

            self.obj().init();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for FretboardBarreSpin {}
}

glib::wrapper! {
    pub struct FretboardBarreSpin(ObjectSubclass<imp::FretboardBarreSpin>)
        @extends gtk::Widget;
}

impl Default for FretboardBarreSpin {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl FretboardBarreSpin {
    pub fn new() -> Self {
        Self::default()
    }

    fn init(&self) {
        self.bind_property("value", &self.imp().label.get(), "label")
            .sync_create()
            .build();

        self.bind_property("value", &self.imp().increment_button.get(), "sensitive")
            .transform_to(|_, value: u8| Some(value < MAX_VALUE))
            .sync_create()
            .build();

        self.bind_property("value", &self.imp().decrement_button.get(), "sensitive")
            .transform_to(|_, value: u8| Some(value > MIN_VALUE))
            .sync_create()
            .build();

        self.imp().increment_button.connect_clicked(glib::clone!(
            #[weak(rename_to = spin)]
            self,
            move |_| {
                spin.set_value(spin.value() + 1);
                spin.emit_by_name::<()>("user-changed-value", &[&"increment"]);
            }
        ));

        self.imp().decrement_button.connect_clicked(glib::clone!(
            #[weak(rename_to = spin)]
            self,
            move |_| {
                spin.set_value(spin.value() - 1);
                spin.emit_by_name::<()>("user-changed-value", &[&"decrement"]);
            }
        ));
    }
}
