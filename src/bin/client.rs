//importing necessary libraries

use cursive::{
    align::HAlign, event::Key, theme::{BaseColor, BorderStyle, Color, Palette, PaletteColor, Theme}, traits::*, view::ScrollStrategy, views::{Dialog, DummyView, EditView, LinearLayout, Panel, ScrollView, TextView}, Cursive
};

use tokio::{
    net::TcpStream,
    sync::Mutex,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}
};

use serde::{Deserialize, Serialize};

use chrono::Local;

use std::{env, sync::Arc, error::Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage{
    username: String,
    content: String,
    timestamp: String,
    message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    UserMessage,
    SystemNotification,
}


// main asynchronous entry point for the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // fetching the username from the command line arguments
    let username = env::args().nth(1).expect("Please provide a username as argument");

    // creating the cursive object
    let mut siv = cursive::default();
    siv.set_theme(create_retro_theme());

    // creating the header to display the chat title and the username
    let header = TextView::new(format!(r#"╔═ RETRO CHAT ═╗ User: {} ╔═ {} ═╗"#, 
        username, 
        Local::now().format("%H:%M:%S")
        ))
        .style(Color::Light(BaseColor::Green))
        .h_align(HAlign::Center);

    // creating scrollable view for message area
    let messages = TextView::new("")
        .with_name("messages")
        .min_height(20)
        .scrollable();

    let messages = ScrollView::new(messages)
        .scroll_strategy(ScrollStrategy::StickToBottom)
        .min_width(60)
        .full_width();

    // creating the input area
    let input = EditView::new()
        .on_submit(move |s, text| send_message(s, text.to_string()))
        .with_name("input")
        .max_height(3)
        .min_width(50)
        .full_width();

    // creating help text for user commands
    let help_text = TextView::new("ESC:Quit | Enter:Send | commands: /help, /clear, /quit")
        .style(Color::Dark(BaseColor::White));

    // creating the main layout
    let layout = LinearLayout::vertical()
        .child(Panel::new(header))
        .child(
        Dialog::around(messages)
                .title("Messages")
                .title_position(HAlign::Center)
                .full_width()
        )
        .child(
        Dialog::around(input)
                .title("Message")
                .title_position(HAlign::Center)
                .full_width()
        )
        .child(Panel::new(help_text).full_width());

    let centered_layout = LinearLayout::horizontal()
        .child(DummyView.full_width())
        .child(layout)
        .child(DummyView.full_width());
      
    siv.add_fullscreen_layer(centered_layout);

    // adding global key bindings
    siv.add_global_callback(Key::Esc, |s| s.quit());

    siv.add_global_callback('/', |s| {
        s.call_on_name("input", |view: &mut EditView| {
            view.set_content("/"); // insert "/" in input box
        });
    });

    // establishing a connection with the server

    let stream = TcpStream::connect("127.0.0.1:8082").await?;
    let (reader, mut writer) = stream.into_split();
    writer.write_all(format!("{}\n", username).as_bytes()).await?;
    
    let writer = Arc::new(Mutex::new(writer));
    let writer_clone = Arc::clone(&writer);
    siv.set_user_data(writer);

    let reader = BufReader::new(reader);
    let mut lines = reader.lines();
    let sink = siv.cb_sink().clone();

    // spawn async task to handle the incoming messages
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(msg) = serde_json::from_str::<ChatMessage>(&line) {
                // Format incoming message based on type
                let formatted_msg = match msg.message_type {
                    MessageType::UserMessage => format!("┌─[{}]\n└─ {} : {}\n",
                        msg.timestamp, msg.username, msg.content),
                    MessageType::SystemNotification => format!("\n[{} {}]\n",
                        msg.username, msg.content),
                };
                // Update UI with the new message
                if sink.send(Box::new(move |siv: &mut Cursive| {
                    siv.call_on_name("messages", |view: &mut TextView| {
                        view.append(formatted_msg); // Append the message
                    });
                })).is_err() {
                    break; // Exit loop on error
                }
            }
        }
    });

    siv.run(); // Run the Cursive event loop
    let _ = writer_clone.lock().await.shutdown().await; // Close the writer
    Ok(()) // Exit successfully

}


//send message function
fn send_message(siv: &mut Cursive, msg: String) {
    if msg.is_empty() { // Ignore empty messages
        return;
    }

    // Handle specific commands
    match msg.as_str() {
        "/help" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append("\n=== Commands ===\n/help - Show this help\n/clear - Clear messages\n/quit - Exit chat\n\n");
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content("");
            });
            return;
        }
        "/clear" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.set_content(""); // Clear messages
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content(""); // Clear input
            });
            return;
        }
        "/quit" => {
            siv.quit(); // Quit the application
            return;
        }
        _ => {}
    }

    // Send the message to the server
    let writer = siv.user_data::<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>().unwrap().clone();
    tokio::spawn(async move {
        let _ = writer.lock().await.write_all(format!("{}\n", msg).as_bytes()).await;
    });

    // Clear the input field
    siv.call_on_name("input", |view: &mut EditView| {
        view.set_content("");
    });
}


//create theme function
fn create_retro_theme() -> Theme {

    let mut theme = Theme::default();
    theme.shadow = true;
    theme.borders = BorderStyle::Simple;

    let mut palette = Palette::default();
    palette[PaletteColor::Background] = Color::Rgb(0, 0, 20); // Dark blue
    palette[PaletteColor::View] = Color::Rgb(0, 0, 20); // Dark blue
    palette[PaletteColor::Primary] = Color::Rgb(0, 255, 0); // Green
    palette[PaletteColor::TitlePrimary] = Color::Rgb(0, 255, 128); // Light green
    palette[PaletteColor::Secondary] = Color::Rgb(255, 191, 0); // Yellow
    palette[PaletteColor::Highlight] = Color::Rgb(0, 255, 255); // Cyan
    palette[PaletteColor::HighlightInactive] = Color::Rgb(0, 128, 128); // Dark cyan
    palette[PaletteColor::Shadow] = Color::Rgb(0, 0, 40); // Darker blue
    theme.palette = palette;

    theme

}