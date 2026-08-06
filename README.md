# zed-agent-launcher 🚀

An interactive CLI launcher designed as a helper for **Zed Editor**'s Terminal Threads init command (`terminal_init_command`).

When launching a new terminal thread in Zed, `zed-agent-launcher` presents an interactive menu to select your desired AI coding agent (`pi`, `Google Antigravity (agy)`, `OpenAI Codex`, `OpenCode`, etc.) and seamlessly replaces the shell process (`exec`) with the selected agent.

---

## ⚡ Quick Start: Zed Integration

Configure `zed-agent-launcher` directly in your Zed settings (`~/.config/zed/settings.json` or `cmd+,` on macOS):

```json
{
  "agent": {
    "terminal_init_command": "s=\"${XDG_CACHE_HOME:-$HOME/.cache}/zed-agent-launcher/run.sh\"; [ -f \"$s\" ] || (mkdir -p \"${s%/*}\" && curl -fsSL https://raw.githubusercontent.com/living42/zed-agent-launcher/main/run.sh -o \"$s\" && chmod +x \"$s\"); \"$s\""
  }
}
```

### Why this One-Liner?
- **Zero Network Overhead**: On the first execution, it downloads and caches the launcher script locally. Subsequent terminal threads launch instantly with **0 network requests**.
- **Auto-Updating**: Periodically checks for new releases in the background (at most once every 24 hours).
- **No Manual Installation Needed**: Downloads pre-built binary matching your OS (macOS / Linux) and architecture (Apple Silicon / Intel / ARM64).

---

## 🛠️ Usage & Options

### Running via Terminal
You can also run `run.sh` directly in your shell:

```bash
# Run latest version
curl -fsSL https://raw.githubusercontent.com/living42/zed-agent-launcher/main/run.sh | sh

# Pin to a specific release version
curl -fsSL https://raw.githubusercontent.com/living42/zed-agent-launcher/main/run.sh | sh -s -- -v v0.1.0

# Force immediate update check
./run.sh --update
```

### Options for `run.sh`
- `-v, --version <TAG>`: Pin a specific version tag to run (e.g. `-v v0.1.0`).
- `-u, --update`: Force an immediate update check against GitHub Releases.
- `-h, --help-wrapper`: Display wrapper help options.

---

## ⚙️ Building from Source

If you prefer to compile `zed-agent-launcher` locally using Rust:

```bash
git clone https://github.com/living42/zed-agent-launcher.git
cd zed-agent-launcher
cargo build --release
```

The compiled binary will be placed at `target/release/zed-agent-launcher`. You can copy it to your `$PATH` (e.g., `/usr/local/bin/zed-agent-launcher`) and configure Zed settings:

```json
{
  "agent": {
    "terminal_init_command": "zed-agent-launcher"
  }
}
```

---

## 📄 License

MIT
