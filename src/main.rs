/**
    tray (c) dotHQ 2021
    A standalone tray application

    The entry point for the tray application

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/
// extern crate gio;
extern crate gtk;
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use dbus::blocking::Connection;

use gtk::{prelude::*, Builder};

// Import glade file to a constant
const LAYOUT: &str = include_str!("tray.glade");

mod status_notifier_host;
mod status_notifier_watcher;

fn main() -> Result<(), Box<dyn Error>> {
    gtk::init().expect("Failed to initialize GTK");

    // Create the UI from a glade file
    let builder = Builder::from_string(LAYOUT);
    let window: gtk::Window = builder.get_object("main_window").unwrap();

    // Make the window larger
    window.resize(200, 200);

    // Show window to users
    window.show_all();

    // Variable to lock until has initied
    let init = Arc::new(Mutex::new(false));

    // Spawn StatusNotifierWatcher (runs forever)
    let watcher_init = init.clone();
    let status_watcher =
        thread::spawn(move || status_notifier_watcher::run(&watcher_init).unwrap());

    thread::sleep(Duration::new(1, 0));

    // Wait until the status watcher has been created
    let _ = init.lock().unwrap();

    // Create a new connection that is going to be used for the host
    let host_connection = Connection::new_session()?;

    // Create the host
    let host = status_notifier_host::StatusNotifierHost::new(&host_connection)?;
    println!("{:?}", host.get_item(0));
    println!("{:?}", host.get_item(1));

    gtk::main();

    // Wait until the status watcher thread has finished
    status_watcher.join().unwrap();

    Ok(())
}
