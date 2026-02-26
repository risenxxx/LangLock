# LangLock

A lightweight, native Windows background utility that intercepts the Caps Lock key to switch the system input language.

## Why LangLock?

Traditional keyboard remapping solutions that emulate virtual keystrokes (via `SendInput`) can be problematic:
- **Anti-cheat detection** — many games detect and block keystroke emulation
- **Modifier key conflicts** — ghost key presses and inconsistent behavior during fast typing

LangLock takes a different approach: it sends `WM_INPUTLANGCHANGEREQUEST` directly to the window instead of emulating keystrokes. This is the same mechanism Windows uses internally when you press `Alt+Shift`.

### Anti-Cheat Considerations

LangLock uses a low-level keyboard hook (`WH_KEYBOARD_LL`) — a standard Windows API also used by many legitimate programs (input methods, screenshot tools, accessibility software). While hooks are generally safe, some aggressive anti-cheat systems may flag any software using them. LangLock minimizes risk by:
- **Not emulating keystrokes** — no `SendInput` calls
- **Ignoring injected events** — filters out `LLKHF_INJECTED` flag
- **Using standard Windows APIs** — only `PostMessageW` with documented messages

## Comparison with Alternatives

| Feature | LangLock | [capslang](https://github.com/edanko/capslang) | [CapsWitch](https://github.com/Linkerin/capswitch) |
|---------|----------|----------|-----------|
| Switch method | `WM_INPUTLANGCHANGEREQUEST` | `WM_INPUTLANGCHANGEREQUEST` | `SendInput` (Win+Space) |
| Keystroke emulation | No | No | **Yes** |
| Shift+Caps Lock = Caps | ✓ (optional) | ✓ | ✓ (via SendInput) |
| Tray icon | ✓ | ✗ | ✓ |
| Works with admin apps | ✓ | ✓ | ✗ |
| Filter injected keys | ✓ | ✗ | ✗ |
| Installer | Inno Setup | PowerShell script | MSIX (self-signed) |
| License | MIT | GPLv3 | MIT |

## How It Works

1. **Low-level keyboard hook** (`WH_KEYBOARD_LL`) intercepts Caps Lock
2. **Swallows the keypress** completely (returns `1` from hook)
3. **Sends `WM_INPUTLANGCHANGEREQUEST`** directly to the foreground window
4. No virtual keystrokes are ever generated

This approach is invisible to games and anti-cheat systems because it never simulates keyboard input.

## Features

- **System tray icon** with context menu
- **Run on startup** option (uses Task Scheduler for elevated privileges)
- **Shift+Caps Lock** for regular Caps Lock behavior (optional, toggle in tray menu)
- **Hide tray icon** to minimize clutter (relaunch to restore)
- **Single instance** enforcement with IPC
- **Zero dependencies** — single portable executable

## Limitations

### Login Screen

LangLock does **not** work on the Windows login screen (password entry). This is a deliberate limitation:

- The login screen runs in a separate Windows session (Session 0)
- Keyboard hooks are session-specific and cannot cross session boundaries
- Implementing this would require a kernel-mode driver or Windows Service with complex cross-session communication

**Workaround:** Use the standard Windows language switcher (`Win+Space` or `Alt+Shift`) on the login screen.

## Installation

### Installer (Recommended)

Download the latest `langlock-setup-*.exe` from [Releases](https://github.com/risenxxx/langlock/releases).

The installer:
- Requires administrator privileges
- Installs to `C:\Program Files\LangLock`
- Optionally adds to Windows startup
- Creates Start Menu shortcuts
- Removes the scheduled task on uninstall

### Portable

Download `langlock.exe` from [Releases](https://github.com/risenxxx/langlock/releases) and run it directly.

## Usage

1. Run LangLock
2. Press **Caps Lock** to switch input language
3. Right-click the tray icon for options:
   - **Run on startup** — Enables auto-start on login
   - **Shift+Caps Lock = regular Caps Lock** — When enabled, pressing Shift+Caps Lock toggles Caps Lock as usual
   - **Hide tray icon** — Hides the tray icon (relaunch to restore)
   - **Exit** — Closes LangLock

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable, MSVC toolchain)
- Windows 10/11
- [Inno Setup 6](https://jrsoftware.org/isinfo.php) (for building installer)

### Build

```bash
git clone https://github.com/risenxxx/langlock.git
cd langlock
cargo build --release
```

The binary will be at `target/release/langlock.exe`.

### Build Installer

```bash
# Requires Inno Setup 6 installed
iscc /DMyAppVersion="0.2.0" installer/langlock.iss
```

## Technical Details

### Why Task Scheduler for Startup?

LangLock uses Windows Task Scheduler instead of the registry `Run` key because:

1. **UIPI (User Interface Privilege Isolation)** — Low-level keyboard hooks need elevated privileges to intercept keystrokes in admin windows and games
2. **UAC-free startup** — Tasks with "Run with highest privileges" don't trigger UAC prompts on every boot
3. **Works with games** — Games running as Administrator receive the hook properly

### Hook Architecture

```
Caps Lock pressed
       ↓
WH_KEYBOARD_LL hook
       ↓
Check: VK_CAPITAL && not injected?
       ↓ Yes
Check: Shift held && feature enabled?
       ↓ Yes → Pass through (normal Caps Lock)
       ↓ No
PostMessage(hwnd, WM_INPUTLANGCHANGEREQUEST, INPUTLANGCHANGE_FORWARD, 0)
       ↓
Return 1 (swallow keypress)
```

## License

MIT License — see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
