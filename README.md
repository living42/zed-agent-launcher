# zed-agent-launcher 🚀

An interactive CLI launcher designed as a helper for **Zed Editor**'s Terminal Threads init command (`terminal_init_command`).

When launching a new terminal thread in Zed, `zed-agent-launcher` presents an interactive menu to select your desired AI coding agent (`pi`, `Google Antigravity (agy)`, `OpenAI Codex`, `OpenCode`) or open a system `shell` session.

---

## ⚡ Quick Start: Zed Integration

Configure `zed-agent-launcher` directly in your Zed settings (`~/.config/zed/settings.json` or `cmd+,` on macOS):

```json
{
  "agent": {
    "terminal_init_command": "exec sh -c 's=\"${XDG_CACHE_HOME:-$HOME/.cache}/zed-agent-launcher/run.sh\"; [ -f \"$s\" ] || (mkdir -p \"${s%/*}\" && curl -fsSL https://raw.githubusercontent.com/living42/zed-agent-launcher/main/run.sh -o \"$s\" && chmod +x \"$s\"); exec \"$s\"'"
  }
}
```

### Options for `run.sh`
- `-v, --version <TAG>`: Pin a specific version tag to run (e.g. `-v v0.1.0`).
- `-u, --update`: Force an immediate update check against GitHub Releases.
- `-h, --help-wrapper`: Display wrapper help options.

---

## 📄 License

MIT
