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
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use dbus::blocking::Connection;

use gtk::{prelude::*, Builder, ButtonBuilder, IconSize, Image};
use status_notifier_host::StatusNotifierItem;

// Import glade file to a constant
const LAYOUT: &str = include_str!("tray.glade");

// mod interface;
mod interfaces;
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

    // Wait until the status watcher has been created
    let _ = init.lock().unwrap();

    // Wait a bit longer for stuff to register
    thread::sleep(Duration::from_millis(10));

    // Create a new connection that is going to be used for the host
    let host_connection: &'static mut Connection = Box::leak(Box::new(Connection::new_session()?));

    // Create the host
    let host = status_notifier_host::StatusNotifierHost::new(&host_connection)?;

    // Here is the important stuff. We are going to grab all of the app indicators
    // from the host and add them to an array
    let app_indicators: &'static mut Vec<StatusNotifierItem> = Box::leak(Box::new(Vec::new()));
    for indicator in host.get_registered_status_notifier_items()? {
        app_indicators.push(StatusNotifierItem::new(indicator, host_connection)?);
    }

    // Now that we have all of the active status notifiers, we need to do a few things with them:
    // 1. Get all of the information we need to display them
    // 2. Add them to the gtk application
    // 3. Setup watchers to check for changes to the app indicators
    // We are going to do most of these things in a separate function
    add_app_indicators(app_indicators, &builder)?;

    gtk::main();

    // Wait until the status watcher thread has finished
    status_watcher.join().unwrap();

    Ok(())
}

fn add_app_indicators<'a>(
    app_indicators: &'static Vec<StatusNotifierItem>,
    builder: &Builder,
) -> Result<(), Box<dyn Error + 'a>> {
    // Get the box that all of the app indicators will be contained in
    let container: gtk::Box = builder.get_object("items").unwrap();

    // Remove the current contents of the box
    for item in container.get_children() {
        container.remove(&item);
    }

    // Loop through each app indicator
    for app in app_indicators {
        let status = app.get_status()?;

        // We will not display app indicator that are in a passive state
        if status == "Passive" {
            continue;
        }

        let theme_path = app.get_icon_theme_path()?;

        // This will be the icon of the app indicator
        let image;

        if status == "Active" {
            let icon_name = app.get_icon_name()?;

            if theme_path != "" {
                // If there is a theme path, you use that

                let image_path = PathBuf::from(&format!("{}/{}.png", theme_path, icon_name));
                println!("{:?}", image_path);
                image = Image::from_file(image_path);
            } else {
                // Otherwise use an icon name

                image = Image::from_icon_name(Some(&icon_name), IconSize::SmallToolbar);
            }
        } else {
            // Must be NeedsAttention
            let icon_name = app.get_attention_icon_name()?;

            if theme_path != "" {
                // If there is a theme path, you use that

                let image_path = PathBuf::from(&format!("{}/{}.png", theme_path, icon_name));
                println!("{:?}", image_path);
                image = Image::from_file(image_path);
            } else {
                // Otherwise use an icon name

                image = Image::from_icon_name(Some(&icon_name), IconSize::SmallToolbar);
            }
        }

        // Create the button for the app indicator
        let button = ButtonBuilder::new().image(&image).visible(true).build();

        // Create a new instance of the App for click events
        let app_indicator = app.to_owned();
        button.connect_clicked(move |a| {
            println!("Click");
            app_indicator.secondary_activate(0, 0).unwrap();
        });

        // Add button to the window
        container.pack_start(&button, false, false, 0);
    }

    Ok(())
}
