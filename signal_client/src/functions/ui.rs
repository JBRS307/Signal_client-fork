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

use crate::functions::contacts::sync_and_print_contacts;
use crate::functions::received::{receive_and_store_messages, show_messages, get_contact_messages};
use crate::functions::sending::{send_message, initialize_app_data};
use crate::functions::contacts::{sync_and_get_contacts, find_name};
use crate::functions::messages::format_timestamp;
use crate::App;


use presage::libsignal_service::ServiceAddress;
// use presage::proto::DataMessage;
use crate::functions::messages::extract_message_info;
use futures::StreamExt;
use presage::manager::ReceivingMode;
use std::ops::RangeFull;
use presage::Manager;
use presage_store_sled::{MigrationConflictStrategy, OnNewIdentity, SledStore};
use chrono::NaiveDateTime;

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

pub async fn start_tui() -> Result<(), Box<dyn std::error::Error>> {
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

    let res = run_app(&mut terminal, &mut app).await;

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
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    let mut input_mode = InputMode::Normal;

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

            let input_paragraph = Paragraph::new(input.as_ref() as &str)
                .block(Block::default().borders(Borders::ALL).title("Input"))
                .style(match input_mode {
                    InputMode::Normal => Style::default().fg(Color::White),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                });
            f.render_widget(input_paragraph, chunks[1]);
            if input_mode == InputMode::Editing {
                f.set_cursor(chunks[1].x + input.len() as u16 + 1, chunks[1].y + 1);
            }
        })?;

        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match input_mode {
                    InputMode::Normal => match key.code {
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
                        KeyCode::Char('e') => {
                            input_mode = InputMode::Editing;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if let Some(selected) = app.selected_contact {
                                let recipient = &app.contacts[selected];
                                let mut arguments = vec![
                                    String::from("send"),
                                    String::from("send"),
                                    recipient.to_string(),
                                ];

                                arguments.push(input.clone());
                                println!("{}", recipient);
                                println!("{}", arguments[3]);
                                if let Err(err) = send_message(arguments).await {
                                    eprintln!("Error sending message: {:?}", err);
                                }
                                println!("test1");
                                input.clear();
                                println!("test2");
                                app.messages = get_contact_messages_with_dates(&app.contacts[selected]).await?;
                            }
                            input_mode = InputMode::Normal;
                        }
                        // KeyCode::Enter => {
                        //     if let Some(selected) = app.selected_contact {
                        //         let recipient = &app.contacts[selected];
                        //         let arguments = vec![
                        //             String::from("send"),
                        //             String::from(recipient),
                        //             input.clone(),
                        //         ];
                        //         if let Err(err) = send_message(arguments).await {
                        //             eprintln!("Error sending message: {:?}", err);
                        //         }
                        //         input.clear();
                        //         app.messages = get_contact_messages_with_dates(&app.contacts[selected]).await?;
                        //     }
                        //     input_mode = InputMode::Normal;
                        // }
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Esc => {
                            input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

// pub async fn start_tui() -> Result<(), Box<dyn std::error::Error>> {
//     // Terminal initialization
//     enable_raw_mode()?;
//     let mut stdout = io::stdout();
//     execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//     let backend = CrosstermBackend::new(stdout);
//     let mut terminal = Terminal::new(backend)?;
//
//     let (contacts, name) = initialize_app_data().await?;
//
//     let mut app = App::new(contacts, name);
//
//     if !app.contacts.is_empty() {
//         app.messages = get_contact_messages_with_dates(&app.contacts[0]).await?;
//     }
//
//     // Run the app
//     let res = run_app(&mut terminal, app).await;
//
//     // Restore terminal
//     disable_raw_mode()?;
//     execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
//     terminal.show_cursor()?;
//
//     if let Err(err) = res {
//         println!("{:?}", err)
//     }
//
//     Ok(())
// }
//
// async fn run_app<B: ratatui::backend::Backend>(
//     terminal: &mut Terminal<B>,
//     mut app: App,
// ) -> io::Result<()> {
//     loop {
//         terminal.draw(|f| {
//             let chunks = Layout::default()
//                 .direction(Direction::Vertical)
//                 .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
//                 .split(f.size());
//
//             let main_chunks = Layout::default()
//                 .direction(Direction::Horizontal)
//                 .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
//                 .split(chunks[0]);
//
//             let contact_items: Vec<ListItem> = app
//                 .contacts
//                 .iter()
//                 .map(|c| ListItem::new(c.as_str()))
//                 .collect();
//
//             let contacts_list = List::new(contact_items)
//                 .block(Block::default().borders(Borders::ALL).title("Contacts"))
//                 .highlight_style(Style::default().bg(Color::LightBlue));
//
//             f.render_widget(contacts_list, main_chunks[0]);
//
//             let messages: Vec<ListItem> = app.messages.iter().map(|(sender, message, _, date)| {
//                 let style = if app.name.as_str() == sender {
//                     Style::default().fg(Color::Green)
//                 } else {
//                     Style::default().fg(Color::Blue)
//                 };
//                 ListItem::new(format!("{} - {}", date, message)).style(style)
//             }).collect();
//
//             let messages_title = match app.selected_contact {
//                 Some(index) => format!("Messages - {}", app.contacts[index]),
//                 None => format!("Messages - {}", app.contacts[0]),
//             };
//
//             let messages_list = List::new(messages)
//                 .block(Block::default().borders(Borders::ALL).title(messages_title.as_str()));
//             f.render_widget(messages_list, main_chunks[1]);
//         })?;
//
//         if crossterm::event::poll(Duration::from_millis(200))? {
//             if let Event::Key(key) = event::read()? {
//                 match key.code {
//                     KeyCode::Char('q') => return Ok(()),
//                     KeyCode::Down => {
//                         if let Some(selected) = app.selected_contact {
//                             if selected < app.contacts.len() - 1 {
//                                 app.selected_contact = Some(selected + 1);
//                             }
//                         } else {
//                             app.selected_contact = Some(0);
//                         }
//                         if let Some(selected) = app.selected_contact {
//                             match get_contact_messages_with_dates(&app.contacts[selected]).await {
//                                 Ok(messages) => app.messages = messages,
//                                 Err(err) => eprintln!("Error fetching messages: {:?}", err),
//                             }
//                         }
//                     }
//                     KeyCode::Up => {
//                         if let Some(selected) = app.selected_contact {
//                             if selected > 0 {
//                                 app.selected_contact = Some(selected - 1);
//                             }
//                         }
//                         if let Some(selected) = app.selected_contact {
//                             match get_contact_messages_with_dates(&app.contacts[selected]).await {
//                                 Ok(messages) => app.messages = messages,
//                                 Err(err) => eprintln!("Error fetching messages: {:?}", err),
//                             }
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }
// }

async fn get_contact_messages_with_dates(contact: &str) -> Result<Vec<(String, String, u64, String)>, Box<dyn std::error::Error>> {
    let messages = get_contact_messages(contact).await?;
    let messages_with_dates = messages.into_iter().map(|(sender, message, timestamp)| {
        let date_string = format_timestamp(timestamp as u64);
        (sender, message, timestamp, date_string)
    }).collect();
    Ok(messages_with_dates)
}

//Moja propozycja
/*
pub async fn receive_and_store_messages_2() -> Result<(), Box<dyn std::error::Error>>  {
    let store = SledStore::open("./registration/main", MigrationConflictStrategy::BackupAndDrop, OnNewIdentity::Trust)?;
    let mut manager = Manager::load_registered(store.clone()).await?;
    let mut messages = Box::pin(manager.receive_messages(ReceivingMode::Forever).await?);

    while let Some(message) = messages.next().await {
        if let Some(selected) = app.selected_contact {
            match get_contact_messages_with_dates(&app.contacts[selected]).await {
                Ok(messages) => app.messages = messages,
                Err(err) => eprintln!("Error fetching messages: {:?}", err),
            }
        }
    }
    Ok(())
}
*/

//propozycja czata
/*
// Function to receive and store messages using a sender
async fn receive_and_store_messages_with_sender(tx: mpsc::Sender<()>) {
    loop {
        // Simulate receiving messages
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Received a new message!");

        // Send a notification to the channel
        if let Err(e) = tx.send(()).await {
            eprintln!("Failed to send message: {}", e);
        }

        // Access and modify the global app state
        let mut app = APP.lock().await;
        app.messages.push(("sender".to_string(), "New message".to_string(), 0, "timestamp".to_string()));
    }
}
*/
