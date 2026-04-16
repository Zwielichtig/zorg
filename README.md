# 📦 Zorg - SSH Manager & Orchestrator

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

Introducing **Zorg**, a TUI application designed to make managing and orchestrating SSH connections effortless and fast. 

Zorg provides a snappy, reliable experience directly in the terminal, eliminating the need to memorize complex SSH connection strings or rely on clunky GUIs.

## ✨ Key Features

- **🚀 Lightning Fast TUI:** Built in Rust with [Ratatui](https://github.com/ratatui/ratatui) for an instantaneous, terminal-native experience
- **🔍 Fuzzy Search:** Instantly find connections by name, username, or hostname with real-time fuzzy matching
- **🗃️ Built-in Database:** Uses a self-contained SQLite database that is automatically managed to store connections securely, with zero setup required
- **💻 Seamless Terminal Integration:** SSH authentication prompts and standard interactive sessions, as well as proxy jumps work flawlessly, feeling just like a native shell

## 🚀 Usage

Using Zorg is as simple as launching the binary in your terminal.

```bash
zorg
```

*From the main interface, use the arrow keys to browse your connections, start typing to fuzzy search, and hit `Enter` to connect!*

## ⬇️ Installation

> [!TIP]
> Get the compiled [release binary](https://github.com/Zwielichtig/zorg/releases)!

### Building from Source

 *Prerequisites*

- [Rust toolchain](https://rustup.rs/) (cargo)
- A working `ssh` client and optionally an `ssh-agent` setup

To build Zorg locally, clone the repository and compile the release binary:

```bash
git clone https://github.com/Zwielichtig/zorg.git
cd zorg
cargo build --release
```

After building, you can find the executable at `target/release/zorg`. You can move this to a directory in your `$PATH` (e.g., `/usr/local/bin` or `~/.local/bin`) for easy access.

```bash
cp target/release/zorg ~/.local/bin/
```

## 🛠️ Configuration

Zorg uses a self-contained SQLite database that is automatically initialized on the first run, so no manual configuration or external database setup is necessary to get started.

## 💭 Feedback and Contributing

Feedback, bug reports, and pull requests are highly appreciated! 

> [!IMPORTANT]
> Check out the [roadmap](ROADMAP.md)!

- Did you find a bug? Open an issue.
- Have a feature request? Let's discuss it!
- Want to contribute? Check out the source code and feel free to submit a Pull Request.

---
*Built with ❤️ for terminal power users.*
