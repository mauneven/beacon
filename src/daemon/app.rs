//! A `GApplication` subclass that remembers the activation token GNOME
//! sends with a notification click, so the window it opens gets focus.

use std::cell::RefCell;

use gio::subclass::prelude::*;

mod imp {
    use super::*;

    type Register = Box<dyn Fn(&gio::DBusConnection)>;

    #[derive(Default)]
    pub struct DaemonApp {
        pub token: RefCell<Option<String>>,
        pub on_register: RefCell<Option<Register>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DaemonApp {
        const NAME: &'static str = "BeaconDaemonApp";
        type Type = super::DaemonApp;
        type ParentType = gio::Application;
    }

    impl ObjectImpl for DaemonApp {}

    impl ApplicationImpl for DaemonApp {
        /// Runs before the bus name is taken, so the interface is there
        /// by the time the first call can arrive.
        fn dbus_register(
            &self,
            connection: &gio::DBusConnection,
            object_path: &str,
        ) -> Result<(), glib::Error> {
            self.parent_dbus_register(connection, object_path)?;
            if let Some(register) = self.on_register.borrow().as_ref() {
                register(connection);
            }
            Ok(())
        }

        fn before_emit(&self, platform_data: &glib::Variant) {
            let dict = glib::VariantDict::new(Some(platform_data));
            let token = ["activation-token", "desktop-startup-id"]
                .iter()
                .find_map(|key| {
                    dict.lookup_value(key, Some(glib::VariantTy::STRING))
                        .and_then(|v| v.str().map(str::to_owned))
                });
            if token.is_some() {
                self.token.replace(token);
            }
            self.parent_before_emit(platform_data);
        }
    }
}

glib::wrapper! {
    pub struct DaemonApp(ObjectSubclass<imp::DaemonApp>)
        @extends gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl DaemonApp {
    /// Run with `--gapplication-service` it stays in the background; run
    /// plainly it also opens the window.
    pub fn new(id: &str) -> Self {
        glib::Object::builder()
            .property("application-id", id)
            .build()
    }

    pub fn connect_dbus_register(&self, f: impl Fn(&gio::DBusConnection) + 'static) {
        self.imp().on_register.replace(Some(Box::new(f)));
    }

    pub fn take_activation_token(&self) -> Option<String> {
        self.imp().token.take()
    }
}
