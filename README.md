# Chatti

Chatti is a terminal-based chat application that interfaces with Ollama, providing a unique way to interact with Ollama's language models directly from your command line.

## Features

- Support for various Ollama models
- Configurable API endpoint and model parameters
- Markdown rendering for chat responses

## Prerequisites

Before you begin, ensure you have the following installed:
- Rust (latest stable version)
- Ollama (running and accessible)

## Configuration

Chatti uses a configuration file located at `~/.config/chatti/config.toml`. If this file doesn't exist, the application will create a default one on first run. You can edit this file to change the following settings:

```toml
api_endpoint = "http://localhost:11434/api/chat"
model = "llama3.2"
stream = true
temperature = 0.7
```

- `api_endpoint`: The URL of your Ollama API endpoint
- `model`: The Ollama model you want to use
- `stream`: Whether to use streaming responses (recommended)
- `temperature`: The temperature parameter for text generation (0.0 to 1.0)

## Usage

To start the application, run:

```
cargo run
```

Once the application starts:
- Type your message and press Enter to send it to the Ollama model
- Use the up and down arrow keys to scroll through the chat history
- Press 'q' to quit the application
- Press '?' to display the help menu

## Development

To run tests:

```
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT](https://github.com/nubilfi/chatti/blob/main/LICENSE)

