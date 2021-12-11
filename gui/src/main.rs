use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder};
use client::Client;


fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("ui_mqtt.ui");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.object("main_window").expect("Couldn't get main_window");
    window.set_application(Some(application));
    window.show_all();
}


fn main() {
    // gtk UI setup an run
    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.builder_basics"), Default::default());

    application.connect_activate(build_ui);
    
    application.run();
}