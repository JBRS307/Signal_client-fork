extern crate core;
extern crate qrcodegen;

mod functions;
use std::env;
use crate::functions::accounts::link_account;
use crate::functions::contacts::sync_and_print_contacts;
use crate::functions::received::{receive_and_store_messages, show_messages, get_contact_messages};
use crate::functions::sending::send_message;
use crate::functions::contacts::sync_and_get_contacts;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;
use presage::libsignal_service::ServiceAddress;
use presage::proto::DataMessage;
use crate::functions::messages::extract_message_info;
use futures::StreamExt;
use presage::manager::ReceivingMode;
use std::ops::RangeFull;

// use crossterm::{
//     event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
//     execute,
//     terminal::{
//         disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
//     },
// };
// use std::{error::Error, io};
// use tui::{
//     backend::{Backend, CrosstermBackend},
//     layout::{Constraint, Direction, Layout},
//     style::{Color, Modifier, Style},
//     text::{Line, Span, Text},
//     widgets::{Block, Borders, List, ListItem, Paragraph},
//     Frame, Terminal,
// };
// use tui_input::backend::crossterm::EventHandler;
// use tui_input::Input;

fn print_options(){
    println!("Please use one of the following options:");
    println!("  send <recipient> <message>   - Send a message to a recipient");
    println!("  account <account_name>      - Link an account");
    println!("  receive                     - Receive and store messages");
    println!("  show <contact>              - Show messages for a contact");
    println!("  tui                         - Start the terminal UI");
}

enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    contacts: Vec<String>,
    messages: Vec<(String, String, u64)>,
    input: String,
    selected_contact: Option<usize>,
}

impl App {
    fn new(contacts: Vec<String>) -> App {
        App {
            contacts,
            messages: vec![],
            input: String::new(),
            selected_contact: None,
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("WELCOME TO OUR SIGNAL-CLIENT!\n");
        print_options();
        return Ok(());
    }

    let option = &args[1];
    match option.as_str() {
        "send" => send_message(args).await?,
        "account" => link_account(args).await?,
        "receive" => receive_and_store_messages().await?,
        "show" => show_messages(args).await?,
        "contacts" => sync_and_print_contacts().await?,
        "tui" => start_tui().await?,
        _ => {
            println!("Invalid option!\n");
            print_options();
        }
    };

    Ok(())
}

async fn start_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let contacts = sync_and_get_contacts().await?;
    let mut app = App::new(contacts);
    app.selected_contact = Some(0);
    if (app.contacts.len() > 0){
        app.messages = get_contact_messages(&app.contacts[0]).await?;
    }

    // Run the app
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(f.size());

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[0]);

            let contact_items: Vec<ListItem> = app
                .contacts
                .iter()
                .map(|c| ListItem::new(c.as_str()))
                .collect();

            let contacts_list = List::new(contact_items)
                .block(Block::default().borders(Borders::ALL).title("Contacts"))
                .highlight_style(Style::default().bg(Color::LightBlue));

            f.render_widget(contacts_list, main_chunks[0]);

            let messages: Vec<ListItem> = app.messages.iter().map(|(sender, message, _)| {
                let style = if is_outgoing_message(sender) {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Blue)
                };
                ListItem::new(message.clone()).style(style)
            }).collect();

            let messages_title = match app.selected_contact {
                Some(index) => format!("Messages - {}", app.contacts[index]),
                None => "Messages".to_string(),
            };

            let messages_list = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title(messages_title.as_str()));
            f.render_widget(messages_list, main_chunks[1]);

            // let input = Paragraph::new(app.input.value())
            //     .block(Block::default().borders(Borders::ALL).title("Input"));
            // let input = List::new(messages)
            //     .block(Block::default().borders(Borders::ALL).title("Input"));

            // f.render_widget(input, chunks[1]);
        })?;

        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        if let Some(selected) = app.selected_contact {
                            if selected < app.contacts.len() - 1 {
                                app.selected_contact = Some(selected + 1);
                            }
                        } else {
                            app.selected_contact = Some(0);
                        }
                        if let Some(selected) = app.selected_contact {
                            match get_contact_messages(&app.contacts[selected]).await {
                                Ok(messages) => app.messages = messages,
                                Err(err) => eprintln!("Error fetching messages: {:?}", err),
                            }
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = app.selected_contact {
                            if selected > 0 {
                                app.selected_contact = Some(selected - 1);
                            }
                        }
                        if let Some(selected) = app.selected_contact {
                            match get_contact_messages(&app.contacts[selected]).await {
                                Ok(messages) => app.messages = messages,
                                Err(err) => eprintln!("Error fetching messages: {:?}", err),
                            }
                        }
                    }
                    // KeyCode::Enter => {
                    //     if let Some(selected) = app.selected_contact {
                    //         let contact = app.contacts[selected].clone();
                    //         app.fetch_messages_for_contact(&contact)
                    //             .await
                    //             .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    //     } else if !app.input.is_empty() {
                    //         app.send_current_message()
                    //             .await
                    //             .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    //     }
                    // }
                    // KeyCode::Char(c) => {
                    //     app.input.push(c);
                    // }
                    // KeyCode::Backspace => {
                    //     app.input.pop();
                    // }
                    _ => {}
                }
            }
        }
    }
}

fn is_outgoing_message(sender: &str) -> bool {
    true
}