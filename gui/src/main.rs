use gtk::prelude::*;
use gtk::{ApplicationWindow, Box, Builder, Button, CheckButton, Entry, Grid, TextView};
use client::client::Client;
use gtk::glib;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use chrono::prelude::*;

const MESSAGE_CONNECTION_START: &str = "Connecting to server...";
const MESSAGE_CONNECTION_FAIL: &str = "Connection failed, please check your settings";

const CONN_RETRIES: usize = 5;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("ui_mqtt.ui");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder
        .object("main_window")
        .expect("Couldn't get main_window");

    // if check_connect_secured is set, show credential fields
    let credential_grid: Grid = builder
        .object("credential_grid")
        .expect("Couldn't get credential_grid");
    let credential_checkbox: CheckButton = builder
        .object("check_connect_secured")
        .expect("Couldn't get credentials_checkbox");
    credential_grid.hide();

    credential_checkbox.connect_toggled(move |credential_checkbox| {
        if credential_checkbox.is_active() {
            credential_grid.show();
        } else {
            credential_grid.hide();
        }
    });

    // if check_last_will is set, show last will fields
    let last_will_grid: Grid = builder
        .object("last_will_grid")
        .expect("Couldn't get credential_grid");
    let last_will_checkbox: CheckButton = builder
        .object("check_last_will")
        .expect("Couldn't get credentials_checkbox");
        last_will_grid.hide();

    last_will_checkbox.connect_toggled(move |last_will_checkbox| {
        if last_will_checkbox.is_active() {
            last_will_grid.show();
        } else {
            last_will_grid.hide();
        }
    });

    let connect_text: gtk::Label = builder
        .object("connect_text")
        .expect("Couldn't get connect_text");
    connect_text.hide();

    window.set_application(Some(application));
    window.set_title("MQTT Client");
    window.set_default_size(900, 600);
    window.show();

    // Main objects used for interaction with the client
    let connect_button: Button = builder
        .object("connect_button")
        .expect("Couldn't get connect_button");
    let disconnect_button: Button = builder
        .object("disconnect_button")
        .expect("Couldn't get disconnect_button");
    let publish_button: Button = builder
        .object("publish_button")
        .expect("Couldn't get publish_button");
    let text_view: TextView = builder
        .object("message_view")
        .expect("Couldn't get message_view");
    let subscription_button: Button = builder
        .object("subscription_button")
        .expect("Couldn't get subscription_button");

    // send client message from rx through channel tx
    let (sender_buffer_messages, receiver_buffer_messages) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let sender = Arc::new(Mutex::new(sender_buffer_messages));

    // send messages to handle buffer messages for widget
    let (sender_from_client, receiver_from_client) = channel::<String>();
    let receiver_from_client = Arc::new(Mutex::new(receiver_from_client));

    // creating mqtt client
    let mut client = Client::new();
    client.set_keepalive_interval(20); // set keepalive interval to 2 minutes

    // if clean_session is set then the aplication will clean the session for the client
    let clean_session: CheckButton = builder.object("clean_session").expect(" Couldnt clean_session");

    // wrapping shared resources for thread safety
    let client = Arc::new(Mutex::new(client));
    let builder = Arc::new(Mutex::new(builder));
    let window = Arc::new(Mutex::new(window));
    let application = Arc::new(Mutex::new(application.clone()));

    // Connect Button clicked event
    {
        let client = client.clone();
        let builder = builder.clone();
        let sender = sender.clone();
        let window = window.clone();
        let application = application.clone();
        let receiver_from_client = receiver_from_client.clone();
        // got to another window on connect_button click
        connect_button.connect_clicked(move |_| {
            let window = window.lock().unwrap();
            let mut client = client.lock().unwrap();
            let application = application.lock().unwrap();
            let builder = builder.lock().unwrap();
            // get host and port from text entry
            let host_entry: gtk::Entry = builder
                .object("server_host_entry")
                .expect("Couldn't get host_entry");
            let port_entry: gtk::Entry = builder
                .object("server_port_entry")
                .expect("Couldn't get port_entry");

            let host = host_entry.text().to_string();
            let port = port_entry.text().to_string();
            // get if credentials are needed
            let credentials_checkbox: gtk::CheckButton = builder
                .object("check_connect_secured")
                .expect("Couldn't get credentials_checkbox");

            let last_will_checkbox: gtk::CheckButton = builder.object("check_last_will").expect("Couldn't get check_last_will");
            let last_will_needed = last_will_checkbox.is_active();

            let will_topic: String;
            let will_message: String;
            if last_will_needed {
                // get will topic and will message
                let will_topic_entry: gtk::Entry = builder
                    .object("will_topic_entry")
                    .expect("Couldn't will_topic_entry");
                let will_message_entry: gtk::Entry = builder
                    .object("will_message_entry")
                    .expect("Couldn't get will_message_entry");
                will_topic = will_topic_entry.text().to_string();
                will_message = will_message_entry.text().to_string();
                println!("will_topic: {}", will_topic);
                println!("will_message: {}", will_message);
            } else {
                // connect client without last will testament
                will_topic = "".to_string();
                will_message = "".to_string();
            }

            let credentials_needed = credentials_checkbox.is_active();

            let connect_spinner: gtk::Spinner = builder
                .object("connect_spinner")
                .expect("Couldn't get connect_spinner");

            let username: String;
            let password: String;
            if credentials_needed {
                // get username and password
                let username_entry: gtk::Entry = builder
                    .object("username_entry")
                    .expect("Couldn't get username_entry");
                let password_entry: gtk::Entry = builder
                    .object("password_entry")
                    .expect("Couldn't get password_entry");
                username = username_entry.text().to_string();
                password = password_entry.text().to_string();
                println!("host: {}:{}", host, port);
                println!("credentials: {}", credentials_needed);
                println!("username: {}", username);
                println!("password: {}", password);
            } else {
                // connect client without credentials
                username = "".to_string();
                password = "".to_string();
            }
            // connect client
            connect_spinner.start();
            window.show();

            let rx_client: Arc<Mutex<Receiver<String>>>;
            match client.connect(
                host.clone(),
                port.clone(),
                username.clone(),
                password.clone(),
                clean_session.is_active(),
                will_topic.clone(),
                will_message.clone(),
            ) {
                Ok(rx_out) => {
                    println!("Connected to server");
                    rx_client = rx_out;
                    connect_text.show();
                    connect_text.set_text(MESSAGE_CONNECTION_START);
                    window.show();
                    // set variables of next window
                    let client_id_connection: gtk::Label = builder
                        .object("client_id_connection")
                        .expect("Couldn't get client_id_connection");
                    let server_host_connection: gtk::Label = builder
                        .object("server_host_connection")
                        .expect("Couldn't get server_host_connection");
                    let server_port_connection: gtk::Label = builder
                        .object("server_port_connection")
                        .expect("Couldn't get server_port_connection");
                    let credential_connection: gtk::Label = builder
                        .object("connect_credential_connection")
                        .expect("Couldn't get credential_connection");
                    client_id_connection
                        .set_text(format!("Client ID: {}", client.get_id_client()).as_str());
                    server_host_connection
                        .set_text(format!("Server host: {}", host.clone()).as_str());
                    server_port_connection
                        .set_text(format!("Server port: {}", port.clone()).as_str());
                    if credentials_needed {
                        credential_connection
                            .set_text(format!("Credentials: {}", &credentials_needed).as_str());
                    } else {
                        credential_connection
                            .set_text(format!("Credentials: {}", &credentials_needed).as_str());
                    }
                }
                Err(e) => {
                    println!("Connection failed: {}", e);
                    drop(client);
                    connect_text.show();
                    connect_text.set_text(MESSAGE_CONNECTION_FAIL);
                    window.show();
                    return;
                }
            };

            let mut i: usize = 0;
            loop {
                if client.is_connected() {
                    break;
                }
                if i > CONN_RETRIES {
                    // print something as error on gui
                    connect_spinner.stop();
                    connect_text.set_text(MESSAGE_CONNECTION_FAIL);
                    client.disconnect();
                    break;
                }
                window.show();
                sleep(Duration::from_millis(2000));
                i += 1;
            }
            connect_spinner.stop();
            // if connected, go to another window
            if client.is_connected() {
                connect_text.set_text("Connected to server");
                window.show();
                let window_connection: ApplicationWindow = builder
                    .object("window_connection")
                    .expect("Couldn't get main_window");

                let sender = sender.clone();
                let receiver_from_client = receiver_from_client.clone();
                thread::Builder::new()
                    .name("thread_receiver_messages".to_string())
                    .spawn(move || {
                        loop {
                            thread::sleep(Duration::from_millis(10));
                            let _sender = sender.lock().unwrap();
                            match rx_client.lock().unwrap().recv() {
                                Ok(message) => {
                                    println!("--> Received message: {}", message);
                                    let _ = _sender.send(Message::UpdateBuffer(message));
                                }
                                Err(e) => {
                                    println!("--> Error receiving message: {}", e);
                                    break;
                                }
                            }

                            match receiver_from_client.lock().unwrap().try_recv() {
                                Ok(message) => {
                                    if message == "buffer_clear" {
                                        println!("Clearing buffer on text message widget");
                                        let _ = _sender.send(Message::ClearBuffer());
                                    }
                                }
                                Err(_e) => {
                                }
                            }

                            // Sending fails if the receiver is closed
                        }
                    })
                    .expect("Couldn't create thread_receiver_messages");

                // Clear subscribed topics when clean session is true
                match builder.object::<Box>("subscribe_list_box") {
                    Some(subscribe_list_box) => {
                        if clean_session.is_active() {
                            println!("Clean session is active, removing old subscribed topics");
                            subscribe_list_box.foreach(|child| {
                                subscribe_list_box.remove(child);
                            });
                        }
                    }
                    None => {
                        print!("Couldn't get subscribe_list_box");
                        // window widget subscribe_list_box is not created yet
                    }
                }
    
                window_connection.set_application(Some(&*application));
                window_connection.show();
            }
        });
    }

    
    // receiver for buffer messages events
    receiver_buffer_messages.attach(None, move |msg| {
        // Window Buffer updated
        let buffer = text_view.buffer().unwrap();
        let mut iter = buffer.end_iter();
        match msg {
            Message::UpdateBuffer(text) => {
                buffer.insert(&mut iter, 
                    format!("{} {}",Utc::now().to_rfc3339(), text).as_str()
            );
                buffer.insert(&mut iter, "\n");
            }
            Message::ClearBuffer() => {
                buffer.delete(&mut buffer.start_iter(), &mut buffer.end_iter());
            }
        }
        // Returning false here would close the receiver
        // and have senders fail
        glib::Continue(true)
    });

    // Publish Button clicked event
    {
        let client = client.clone();
        let builder = builder.clone();
        publish_button.connect_clicked(move |_| {
            let builder = builder.lock().unwrap();
            let mut client = client.lock().unwrap();

            let topic_entry: gtk::Entry = builder
                .object("topic_entry")
                .expect("Couldn't get topic_entry");
            let retain_check: gtk::CheckButton = builder
                .object("retain_check")
                .expect("Couldn't get retain_check");
            let message_publish: gtk::Entry = builder
                .object("message_publish")
                .expect("Couldn't get message_publish");
            let qos_entry: Entry = builder.object("qos_entry").expect("Couldn't get qos_entry");
            let qos: u8 = qos_entry
                .text()
                .to_string()
                .parse()
                .expect("Couldn't parse qos");
            let topic = topic_entry.text().to_string();
            let dup = 0;
            let retain = if retain_check.is_active() { 1 } else { 0 };
            let message = message_publish.text().to_string();
            client.publish(qos, dup, retain, &topic.clone(), &message);
        });
    }

    // Subscribe button clicked event
    {
        let client_subscription_events = client.clone();
        let builder = builder.clone();
        let window = window.clone();

        subscription_button.connect_clicked(move |_| {
            let builder = builder.lock().unwrap();
            let mut client = client_subscription_events.lock().unwrap();
            let window = window.lock().unwrap();

            // make new subscribe list box and add it to the window
            let subscribe_list_box: Box = builder
                .object("subscribe_list_box")
                .expect("Couldn't get subscribe_list_box");
            let subscription_entry: Entry = builder
                .object("subscription_entry")
                .expect("Couldn't get subscription_entry");

            let subscription_topic = subscription_entry.text().to_string();

            // send client subscribe request
            client.subscribe(&subscription_topic);

            // create new list box row with the subscribe topic
            let row = gtk::ListBoxRow::new();
            row.set_margin_top(15);
            let hbox = Box::new(gtk::Orientation::Horizontal, 30);
            let label = gtk::Label::new(Some(subscription_topic.as_str()));
            let unsubscribe_topic_x_button = Button::with_label("Unsubscribe"); // this button is the unsubscribe for this topic

            //set_id(subscription_topic.clone());
            row.add(&hbox);
            hbox.pack_start(&label, false, true, 60);
            hbox.pack_start(&unsubscribe_topic_x_button, false, false, 20);
            subscribe_list_box.add(&row);
            subscribe_list_box.show_all();
            window.show();

            let client_unsubscribe_event = client_subscription_events.clone();
            // button unsubscribe clicked event
            unsubscribe_topic_x_button.connect_clicked(move |_| {
                let mut client = client_unsubscribe_event.lock().unwrap();
                client.unsubscribe(&subscription_topic);
                subscribe_list_box.remove(&row);
                subscribe_list_box.show_all();
            });
        });
    }

    // Disconnect button clicked event
    {
        let client = client.clone();
        let builder = builder.clone();
        disconnect_button.connect_clicked(move |_| {
            let builder = builder.lock().unwrap();
            let mut client = client.lock().unwrap();
            let window_connection: ApplicationWindow = builder
                .object("window_connection")
                .expect("Couldn't get main_window");
            client.disconnect();
            window_connection.hide();

            let connect_text: gtk::Label = builder
                .object("connect_text")
                .expect("Couldn't get connect_text");
            connect_text.set_text("");
            if !client.is_connected() {
                connect_text.set_text("Disconnected from server");
                // send clear buffer message to handler of buffer thread
                sender_from_client.send("buffer_clear".to_string()).unwrap();

            }
            // set to empty for text entry
            let username_entry: gtk::Entry = builder
                .object("username_entry")
                .expect("Couldn't get username_entry");
            let password_entry: gtk::Entry = builder
                .object("password_entry")
                .expect("Couldn't get password_entry");
            let credentials_checkbox: gtk::CheckButton = builder
                .object("check_connect_secured")
                .expect("Couldn't get credentials_checkbox");
            let topic_entry: gtk::Entry = builder
                .object("topic_entry")
                .expect("Couldn't get topic_entry");
            let retain_check: gtk::CheckButton = builder
                .object("retain_check")
                .expect("Couldn't get retain_check");
            let message_publish: gtk::Entry = builder
                .object("message_publish")
                .expect("Couldn't get message_publish");
            let subscription_entry: Entry = builder
                .object("subscription_entry")
                .expect("Couldn't get subscription_entry");
            let last_will_checkbox: gtk::CheckButton = builder
                .object("check_last_will")
                .expect("Couldn't get check_last_will");
            let will_topic_entry: gtk::Entry = builder
                .object("will_topic_entry")
                .expect("Couldn't will_topic_entry");
            let will_message_entry: gtk::Entry = builder
                .object("will_message_entry")
                .expect("Couldn't get will_message_entry");

            
            username_entry.set_text("");
            password_entry.set_text("");
            credentials_checkbox.set_active(false);
            topic_entry.set_text("");
            message_publish.set_text("");
            retain_check.set_active(false);
            subscription_entry.set_text("");
            last_will_checkbox.set_active(false);
            will_topic_entry.set_text("");
            will_message_entry.set_text("");
            
        });
    }
}

// Messages enum for receiver thread for the connected screen messages section
enum Message {
    UpdateBuffer(String),
    ClearBuffer(),
}

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let application = gtk::Application::new(Some("mqtt.rustmonnaz.client"), Default::default());

    application.connect_activate(build_ui);

    application.run();
}
