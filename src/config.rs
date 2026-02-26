//! Configuration persistence with portable mode support.
//!
//! Settings are stored in a simple INI-like format (key=value).
//! Location is determined by the presence of a `.portable` marker file:
//! - If `.portable` exists next to the executable: settings stored in exe directory
//! - Otherwise: settings stored in %APPDATA%\LangLock

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Settings file name.
const SETTINGS_FILE: &str = "langlock.settings.ini";

/// Portable mode marker file name.
const PORTABLE_MARKER: &str = ".portable";

/// Configuration keys.
const KEY_SHIFT_CAPS_ENABLED: &str = "shift_caps_lock_enabled";
const KEY_TRAY_HIDDEN: &str = "tray_hidden";

/// Gets the executable directory.
fn get_exe_dir() -> Option<PathBuf> {
    env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// Checks if portable mode is enabled (`.portable` file exists next to exe).
fn is_portable_mode() -> bool {
    get_exe_dir()
        .map(|dir| dir.join(PORTABLE_MARKER).exists())
        .unwrap_or(false)
}

/// Gets the config directory path.
/// - Portable mode: same directory as executable
/// - Normal mode: %APPDATA%\LangLock
fn get_config_dir() -> Option<PathBuf> {
    if is_portable_mode() {
        get_exe_dir()
    } else {
        env::var("APPDATA").ok().map(|p| PathBuf::from(p).join("LangLock"))
    }
}

/// Gets the settings file path.
fn get_settings_path() -> Option<PathBuf> {
    get_config_dir().map(|p| p.join(SETTINGS_FILE))
}

/// Parses INI-like content into a HashMap.
fn parse_ini(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        // Parse key=value
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

/// Parses a boolean value from string.
fn parse_bool(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
}

/// Loads all settings from the config file.
fn load_settings() -> HashMap<String, String> {
    let path = match get_settings_path() {
        Some(p) => p,
        None => return HashMap::new(),
    };

    match fs::read_to_string(&path) {
        Ok(content) => parse_ini(&content),
        Err(_) => HashMap::new(),
    }
}

/// Saves all settings to the config file.
fn save_settings(settings: &HashMap<String, String>) {
    let dir = match get_config_dir() {
        Some(d) => d,
        None => return,
    };

    let path = match get_settings_path() {
        Some(p) => p,
        None => return,
    };

    // Create directory if it doesn't exist (only for non-portable mode)
    if !is_portable_mode() {
        let _ = fs::create_dir_all(&dir);
    }

    // Build content
    let mut content = String::from("# LangLock Settings\n\n");
    for (key, value) in settings {
        content.push_str(&format!("{}={}\n", key, value));
    }

    let _ = fs::write(&path, content);
}

/// Loads the Shift+Caps Lock setting from config file.
/// Returns `true` (enabled) by default if not set.
pub fn load_shift_caps_enabled() -> bool {
    let settings = load_settings();
    settings
        .get(KEY_SHIFT_CAPS_ENABLED)
        .map(|v| parse_bool(v))
        .unwrap_or(true) // Default: enabled
}

/// Saves the Shift+Caps Lock setting to config file.
pub fn save_shift_caps_enabled(enabled: bool) {
    let mut settings = load_settings();
    settings.insert(KEY_SHIFT_CAPS_ENABLED.to_string(), enabled.to_string());
    save_settings(&settings);
}

/// Loads the tray hidden setting from config file.
/// Returns `false` (visible) by default if not set.
pub fn load_tray_hidden() -> bool {
    let settings = load_settings();
    settings
        .get(KEY_TRAY_HIDDEN)
        .map(|v| parse_bool(v))
        .unwrap_or(false) // Default: visible
}

/// Saves the tray hidden setting to config file.
pub fn save_tray_hidden(hidden: bool) {
    let mut settings = load_settings();
    settings.insert(KEY_TRAY_HIDDEN.to_string(), hidden.to_string());
    save_settings(&settings);
}
