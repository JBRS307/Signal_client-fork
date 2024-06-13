# Signal Client

Welcome to our Signal Client! This application allows you to send and receive messages, link accounts, sync contacts, and use a terminal-based user interface.

## Getting Started

To use this application, you will need to have:
- [Rust](https://www.rust-lang.org/) installed on your system
- [Protoc](https://github.com/protocolbuffers/protobuf/releases/tag/v27.0-rc2) installed, appropriate to your system

### Usage

Run the application with the following command:

```bash
cargo run
```

Here are the available options:

- `send <recipient> <message>` - Send a message to a recipient.
- `account <account_name>` - Link an account.
- `receive` - Receive and store messages.
- `contacts` - Show all contacts.
- `show <contact>` - Show messages for a contact.
- `tui` - Start the terminal UI.

### Example Commands

- To send a message:

    ```bash
    cargo run send "My contact" "Hello, World!"
    ```

- To link an account:

    ```bash
    cargo run account my_account_name
    ```

- To receive messages:

    ```bash
    cargo run receive
    ```

- To sync and show contacts:

    ```bash
    cargo run contacts
    ```

- To show messages for a specific contact:

    ```bash
    cargo run show "My contact"
    ```

- To start the terminal-based user interface:

    ```bash
    cargo run tui
    ```

## Project Structure

The project is structured into modules to organize different functionalities:

- `functions/accounts.rs` - Contains functions to link accounts.
- `functions/contacts.rs` - Contains functions to sync and print contacts.
- `functions/received.rs` - Contains functions to receive and show messages.
- `functions/sending.rs` - Contains functions to send messages.
- `functions/ui.rs` - Contains functions to start the terminal user interface.
  
## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.

---
