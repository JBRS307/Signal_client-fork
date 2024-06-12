extern crate core;
extern crate qrcodegen;

mod functions;
use std::env;
use crate::functions::accounts::link_account;
use crate::functions::contacts::sync_and_print_contacts;
use crate::functions::received::{receive_and_store_messages, show_messages, get_contact_messages};
use crate::functions::sending::{send_message, initialize_app_data};
use crate::functions::contacts::{sync_and_get_contacts, find_name};
use crate::functions::messages::format_timestamp;

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
use presage::Manager;
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use chrono::NaiveDateTime;

fn print_options() {
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
    messages: Vec<(String, String, u64, String)>,
    input: String,
    selected_contact: Option<usize>,
    name: String,
}

impl App {
    fn new(contacts: Vec<String>, name: String) -> App {
        App {
            contacts,
            messages: vec![],
            input: String::new(),
            selected_contact: None,
            name,
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

    let (contacts, name) = initialize_app_data().await?;

    let mut app = App::new(contacts, name);

    if !app.contacts.is_empty() {
        app.messages = get_contact_messages_with_dates(&app.contacts[0]).await?;
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

            let messages: Vec<ListItem> = app.messages.iter().map(|(sender, message, _, date)| {
                let style = if app.name.as_str() == sender {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Blue)
                };
                ListItem::new(format!("{} - {}", date, message)).style(style)
            }).collect();

            let messages_title = match app.selected_contact {
                Some(index) => format!("Messages - {}", app.contacts[index]),
                None => format!("Messages - {}", app.contacts[0]),
            };

            let messages_list = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title(messages_title.as_str()));
            f.render_widget(messages_list, main_chunks[1]);
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
                            match get_contact_messages_with_dates(&app.contacts[selected]).await {
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
                            match get_contact_messages_with_dates(&app.contacts[selected]).await {
                                Ok(messages) => app.messages = messages,
                                Err(err) => eprintln!("Error fetching messages: {:?}", err),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn get_contact_messages_with_dates(contact: &str) -> Result<Vec<(String, String, u64, String)>, Box<dyn std::error::Error>> {
    let messages = get_contact_messages(contact).await?;
    let messages_with_dates = messages.into_iter().map(|(sender, message, timestamp)| {
        let date_string = format_timestamp(timestamp as u64);
        (sender, message, timestamp, date_string)
    }).collect();
    Ok(messages_with_dates)
}
