use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder};
use gtk::prelude::EntryExt;
use gtk::prelude::ToggleButtonExt;
use client::client::Client;
use std::thread::sleep;
use std::time::Duration;

const MESSAGE_CONNECTION_START: &str = "Connecting to server...";
const MESSAGE_CONNECTION_FAIL: &str = "Connection failed, please check your settings";

const CONN_RETRIES: usize = 5;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("ui_mqtt.ui");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.object("main_window").expect("Couldn't get main_window");
    
    // if check_connect_secured is set, show credential fields
    let credential_grid: gtk::Grid = builder.object("credential_grid").expect("Couldn't get credential_grid");
    let credential_checkbox: gtk::CheckButton = builder.object("check_connect_secured").expect("Couldn't get credentials_checkbox");
    credential_grid.hide();
    credential_checkbox.connect_toggled(move |credential_checkbox| {
        if credential_checkbox.is_active() {
            credential_grid.show();
        } else {
            credential_grid.hide();
        }
    });

    let connect_text: gtk::Label = builder.object("connect_text").expect("Couldn't get connect_text");
    connect_text.hide();

    window.set_application(Some(application));
    window.show();


    let connect_button: gtk::Button = builder.object("connect_button").expect("Couldn't get connect_button");
    // got to another window on connect_button click
    connect_button.connect_clicked(move |_| {
        let mut client = Client::new();

        // get host and port from text entry
        let host_entry: gtk::Entry = builder.object("server_host_entry").expect("Couldn't get host_entry");
        let port_entry: gtk::Entry = builder.object("server_port_entry").expect("Couldn't get port_entry");

        let host = host_entry.text().to_string();
        let port = port_entry.text().to_string();
        // get if credentials are needed
        let credentials_checkbox: gtk::CheckButton = builder.object("check_connect_secured").expect("Couldn't get credentials_checkbox");

        let credentials_needed = credentials_checkbox.is_active();

        let connect_spinner: gtk::Spinner = builder.object("connect_spinner").expect("Couldn't get connect_spinner");

        let username: String;
        let password: String;
        if credentials_needed {
            // get username and password
            let username_entry: gtk::Entry = builder.object("username_entry").expect("Couldn't get username_entry");
            let password_entry: gtk::Entry = builder.object("password_entry").expect("Couldn't get password_entry");
            username = username_entry.text().to_string();
            password = password_entry.text().to_string();
            println!("host: {}:{}", host, port);
            println!("credentials: {}", credentials_needed);
            println!("username: {}", username);
            println!("password: {}", password);

        } else {
            // connect client without credentials
            username = ' '.to_string();
            password = ' '.to_string();
        }
        // connect client
        connect_spinner.start();
        window.show();
        match client.connect(host.clone(), port.clone(), username.clone(), password.clone()) {
            Ok(_) => {
                println!("Connected to server");
                connect_text.show();
                connect_text.set_text(MESSAGE_CONNECTION_START);
                window.show();
                // set variables of next window
                let server_host_connection: gtk::Label = builder.object("server_host_connection").expect("Couldn't get server_host_connection");
                let server_port_connection: gtk::Label = builder.object("server_port_connection").expect("Couldn't get server_port_connection");
                let credential_connection: gtk::Label = builder.object("connect_credential_connection").expect("Couldn't get credential_connection");
                server_host_connection.set_text(format!("Server host: {}",host.clone()).as_str());
                server_port_connection.set_text(format!("Server port: {}",port.clone()).as_str());
                if credentials_needed {
                    credential_connection.set_text(format!("Credentials: {}",&credentials_needed).as_str());
                } else {
                    credential_connection.set_text(format!("Credentials: {}",&credentials_needed).as_str());
                }
            },
            Err(e) => {
                println!("Connection failed: {}", e);
                connect_text.show();
                connect_text.set_text(MESSAGE_CONNECTION_FAIL);
                window.show();
            }
        }

        let mut i:usize = 0;
        loop {
            if client.is_connected() { break; }
            if i > CONN_RETRIES {
                // println!("--> Not connected to server");
                // print something as error on gui
                connect_spinner.stop();
                connect_text.set_text(MESSAGE_CONNECTION_FAIL);
                break;
            }
            sleep(Duration::from_millis(2000));
            i += 1;
        };
        connect_spinner.stop();
        // if connected, go to another window
        if client.is_connected() {
            connect_text.set_text("Connected to server");
            window.show();
            println!("--> Connected to server");
            let window_connection: ApplicationWindow = builder.object("window_connection").expect("Couldn't get main_window");
            window_connection.show();
        }
    });
}


fn main() {
    // gtk UI setup an run
    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.builder_basics"), Default::default());

    application.connect_activate(build_ui);
    
    application.run();
}