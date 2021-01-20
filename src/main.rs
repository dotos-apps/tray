/**
    dotOS internal app template. Fill out the following lines and delete this one. Add this comment to every line

    <appname> (c) dotHQ 2021
    <app description>

    <file description>

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/
extern crate gtk;
use gtk::{prelude::*, Builder};

// Import glade file to a constant
const LAYOUT: &str = include_str!("layout.glade");

fn main() {
    gtk::init().expect("Failed to initialize GTK");

    // Create the UI from a glade file
    let builder = Builder::from_string(LAYOUT);
    let window: gtk::Window = builder.get_object("main_window").unwrap();

    // Make the window larger
    window.resize(200, 200);

    // Show window to users
    window.show_all();

    gtk::main();
}
