# Rust Async Twitch BOT Project

# This project is currently under development.

This project is a Rust-based asynchronous application that integrates with Twitch and performs text-to-speech (TTS) operations.
It uses several Rust crates including `tokio` async framework.

## Project Structure

- `main.rs`: The entry point of the application.
- `config.rs`: Handles configuration loading.
- `bus.rs`: Implements a message bus for inter-module communication.
- `defs.rs`: Contains various definitions used throughout the project.
- `irc_parser.rs`: Parses IRC messages.
- `tts.rs`: Handles text-to-speech functionality.
- `twitch.rs`: Manages Twitch integration.

## Dependencies

- `async-std`: For asynchronous programming.
- `tokio`: For asynchronous runtime.
- `serde`: For serialization and deserialization.
- `irc-parser`: For parsing IRC messages.

## Configuration

The application expects a configuration file named `config.toml` in the root directory. The configuration file should contain the following:

```toml
[server]
address = "your_server_address"
port = 6667

[user]
name = "your_username"
token = "your_oauth_token"
```

## Example Usage

The application will connect to the specified Twitch server and user, and start listening for messages. It will also perform text-to-speech operations based on the received messages.

If the configuration file is not found or cannot be loaded, the application will print an error message and exit.

License
This project is licensed under the MIT License.

```
This README.md file provides an overview of the project,
its structure, dependencies, configuration, and usage instructions.
Adjust the content as needed to better fit your project's specifics.
```
