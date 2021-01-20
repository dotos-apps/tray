/**
    tray (c) dotHQ 2021
    A standalone tray application

    The entry point for the tray application

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/
// extern crate gio;
// extern crate gtk;
use std::error::Error;

use const_format::concatcp;
use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
// use gio::{
//     bus_own_name, bus_watch_name, BusNameOwnerFlags, BusNameWatcherFlags, BusType, DBusConnection,
//     DBusSignalFlags, WatcherId,
// };
// use gtk::{prelude::*, Builder};

// Import glade file to a constant
// const LAYOUT: &str = include_str!("layout.glade");

// DBus channels
const PREFIX: &str = "org.kde";
const WATCHER_BUS_NAME: &str = concatcp!(PREFIX, ".StatusNotifierWatcher");

#[derive(Clone)]
struct StatusNotifierItem {
    service: String,
    sender: String,
}

impl StatusNotifierItem {
    pub fn to_register_string(&self) -> String {
        format!("{}{}", self.sender, self.service)
    }
}

struct StatusNotifierWatcher {
    pub services: Vec<StatusNotifierItem>,
}

impl StatusNotifierWatcher {
    fn new() -> Self {
        StatusNotifierWatcher {
            services: Vec::new(),
        }
    }

    pub fn services_to_register_string(&self) -> Vec<String> {
        (&self.services)
            .into_iter()
            .map(|sni| sni.to_register_string())
            .collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // gtk::init().expect("Failed to initialize GTK");

    // // Create the UI from a glade file
    // let builder = Builder::from_string(LAYOUT);
    // let window: gtk::Window = builder.get_object("main_window").unwrap();

    // // Make the window larger
    // window.resize(200, 200);

    // // Show window to users
    // window.show_all();

    let connection = Connection::new_session()?;

    // Request the dbus name that is required for AppIndicators
    connection.request_name("org.kde.StatusNotifierWatcher", false, true, false)?;

    // Create a crossroads
    let mut cr = Crossroads::new();

    let status_notifier_watcher = cr.register("org.kde.StatusNotifierWatcher", |b| {
        // Methods
        // -------
        // Register status notifier host
        b.method(
            "RegisterStatusNotifierHost",
            ("service",),
            (),
            |_, _, (service,): (String,)| {
                println!("RegisterStatusNotifierHost service={}", service);
                Ok(())
            },
        );

        // Register status notifier item
        b.method(
            "RegisterStatusNotifierItem",
            ("service",),
            (),
            |context, data: &mut StatusNotifierWatcher, (service,): (String,)| {
                // Log register information to the console
                println!("RegisterStatusNotifierItem service={}", service);

                // Add the service to the data store
                data.services.push(StatusNotifierItem {
                    service: service.clone(),
                    sender: context.message().sender().unwrap().to_string(),
                });

                // Create and send the StatusNotifierItemRegistered signal
                let signal_msg = context.make_signal("StatusNotifierItemRegistered", ("/",));
                context.push_msg(signal_msg);

                // Return
                Ok(())
            },
        );

        // Signals
        // -------
        // On host register
        b.signal::<(), &'static str>("StatusNotifierHostRegistered", ());
        // On host unregister
        b.signal::<(), &'static str>("StatusNotifierHostUnregistered", ());
        // Status notifier item registered
        b.signal::<(&'static str,), &'static str>("StatusNotifierItemRegistered", ("String",));
        // Status notifier item unregistered
        b.signal::<(&'static str,), &'static str>("StatusNotifierItemUnregistered", ("String",));

        // Properties
        // ----------
        // Note: You use `get` and `set` for setting and getting the values
        b.property::<Vec<String>, &str>("RegisteredStatusNotifierItems")
            .get(|_, data| Ok(data.services_to_register_string()));
        b.property::<bool, &str>("IsStatusNotifierHostRegistered")
            .get(|_, _| Ok(true));
        b.property::<u8, &str>("ProtocolVersion").get(|_, _| Ok(0));
    });

    // Insert the functionality into our watcher
    cr.insert(
        "/StatusNotifierWatcher",
        &[status_notifier_watcher],
        StatusNotifierWatcher::new(),
    );

    // Add to the connection
    cr.serve(&connection)?;

    // gtk::main();

    Ok(())
}
