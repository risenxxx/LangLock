//! System tray icon and menu management.

use crate::config;
use crate::hook;
use crate::notification::show_hidden_notification;
use crate::startup;
use muda::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Menu item IDs.
const MENU_STARTUP_ID: &str = "startup";
const MENU_SHIFT_CAPS_ID: &str = "shift_caps";
const MENU_HIDE_ID: &str = "hide";
const MENU_EXIT_ID: &str = "exit";

/// Global flag indicating if exit was requested.
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Manages the system tray icon and its context menu.
///
/// Exactly one [`TrayIcon`] is created per process and kept alive for the whole
/// run; hiding and showing only toggle its visibility. It is never rebuilt.
///
/// Rebuilding is what produced duplicate icons: when LangLock starts at logon it
/// races `explorer.exe`, and `Shell_NotifyIcon(NIM_ADD)` can report failure even
/// though the shell went on to register the icon. A retry then added a second
/// one. `tray-icon` now keeps its hidden window on such a failure and re-registers
/// on the shell's `TaskbarCreated` broadcast, so the single icon shows up on its
/// own once the taskbar is ready — no retry needed on our side.
pub struct TrayManager {
    /// `None` only if the hidden helper window could not be created at all, which
    /// leaves LangLock running headless rather than aborting the keyboard hook.
    tray_icon: Option<TrayIcon>,
    startup_item: CheckMenuItem,
    shift_caps_item: CheckMenuItem,
    /// Mirrors the icon's visibility so repeated calls don't hit the shell.
    visible: bool,
}

impl TrayManager {
    /// Creates the tray icon and its context menu.
    ///
    /// Never fails the caller: if the icon cannot be created, the error is logged
    /// and LangLock keeps running with the keyboard hook only.
    pub fn new() -> Self {
        let startup_item = CheckMenuItem::with_id(
            MENU_STARTUP_ID,
            "Run on startup",
            true,
            startup::is_startup_enabled(),
            None,
        );
        let shift_caps_item = CheckMenuItem::with_id(
            MENU_SHIFT_CAPS_ID,
            "Shift+Caps Lock = regular Caps Lock",
            true,
            hook::is_shift_capslock_enabled(),
            None,
        );
        let hide_item = MenuItem::with_id(MENU_HIDE_ID, "Hide tray icon", true, None);
        let exit_item = MenuItem::with_id(MENU_EXIT_ID, "Exit", true, None);

        let tray_icon = match build_icon(&startup_item, &shift_caps_item, &hide_item, &exit_item) {
            Ok(icon) => Some(icon),
            Err(e) => {
                eprintln!("Failed to create tray icon: {}", e);
                None
            }
        };

        Self {
            tray_icon,
            startup_item,
            shift_caps_item,
            visible: true,
        }
    }

    /// Hides the tray icon and saves the state.
    pub fn hide(&mut self) {
        let _ = self.set_visible(false);
        config::save_tray_hidden(true);
        show_hidden_notification();
    }

    /// Hides the tray icon silently (no notification, used on startup).
    pub fn hide_silently(&mut self) {
        let _ = self.set_visible(false);
    }

    /// Shows/restores the tray icon and saves the state.
    pub fn show(&mut self) -> Result<(), String> {
        // Sync checkboxes with the current settings before the menu is seen again.
        self.startup_item.set_checked(startup::is_startup_enabled());
        self.shift_caps_item
            .set_checked(hook::is_shift_capslock_enabled());

        config::save_tray_hidden(false);
        self.set_visible(true)
    }

    /// Applies the requested visibility to the existing icon.
    fn set_visible(&mut self, visible: bool) -> Result<(), String> {
        if self.visible == visible {
            return Ok(());
        }

        if let Some(tray) = &self.tray_icon {
            tray.set_visible(visible)
                .map_err(|e| format!("Failed to change tray icon visibility: {}", e))?;
        }

        self.visible = visible;
        Ok(())
    }

    /// Toggles the startup state and updates the checkbox.
    pub fn toggle_startup(&self) {
        let currently_enabled = startup::is_startup_enabled();

        if currently_enabled {
            let _ = startup::disable_startup();
        } else {
            let _ = startup::enable_startup();
        }

        // Sync checkbox with actual state (in case UAC was cancelled)
        let new_state = startup::is_startup_enabled();
        self.startup_item.set_checked(new_state);
    }

    /// Toggles the Shift+Caps Lock feature and updates the checkbox.
    pub fn toggle_shift_caps(&self) {
        let currently_enabled = hook::is_shift_capslock_enabled();
        let new_state = !currently_enabled;
        hook::set_shift_capslock_enabled(new_state);
        config::save_shift_caps_enabled(new_state);
        self.shift_caps_item.set_checked(new_state);
    }
}

/// Builds the tray icon and attaches the context menu assembled from the items.
fn build_icon(
    startup_item: &CheckMenuItem,
    shift_caps_item: &CheckMenuItem,
    hide_item: &MenuItem,
    exit_item: &MenuItem,
) -> Result<TrayIcon, String> {
    let menu = Menu::new();
    menu.append(startup_item)
        .map_err(|e| format!("Failed to add startup item: {}", e))?;
    menu.append(shift_caps_item)
        .map_err(|e| format!("Failed to add shift caps item: {}", e))?;
    menu.append(hide_item)
        .map_err(|e| format!("Failed to add hide item: {}", e))?;
    menu.append(&PredefinedMenuItem::separator())
        .map_err(|e| format!("Failed to add separator: {}", e))?;
    menu.append(exit_item)
        .map_err(|e| format!("Failed to add exit item: {}", e))?;

    let icon = create_icon()?;
    TrayIconBuilder::new()
        .with_tooltip("LangLock - Caps Lock Language Switcher")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .map_err(|e| format!("Failed to create tray icon: {}", e))
}

/// Creates the tray icon from embedded RGBA data.
fn create_icon() -> Result<Icon, String> {
    // Create a simple 32x32 icon with an "L" shape
    // Colors: Blue background (#2563eb) with white "L"
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;

            // Background color (blue)
            let (r, g, b) = (37, 99, 235); // #2563eb

            // Draw "L" shape in white
            let is_l = (x >= 8 && x <= 12 && y >= 6 && y <= 24) // Vertical bar
                    || (x >= 8 && x <= 22 && y >= 20 && y <= 24); // Horizontal bar

            if is_l {
                // White
                rgba[idx] = 255;     // R
                rgba[idx + 1] = 255; // G
                rgba[idx + 2] = 255; // B
                rgba[idx + 3] = 255; // A
            } else {
                // Blue background
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }
    }

    Icon::from_rgba(rgba, SIZE, SIZE).map_err(|e| format!("Failed to create icon: {}", e))
}

/// Handles menu events and returns true if exit was requested.
pub fn handle_menu_event(event: MenuEvent, tray: &mut TrayManager) -> bool {
    let id = event.id();

    if *id == MenuId::new(MENU_EXIT_ID) {
        EXIT_REQUESTED.store(true, Ordering::SeqCst);
        return true;
    }

    if *id == MenuId::new(MENU_HIDE_ID) {
        tray.hide();
        return false;
    }

    if *id == MenuId::new(MENU_STARTUP_ID) {
        tray.toggle_startup();
        return false;
    }

    if *id == MenuId::new(MENU_SHIFT_CAPS_ID) {
        tray.toggle_shift_caps();
        return false;
    }

    false
}

/// Checks if exit has been requested.
pub fn is_exit_requested() -> bool {
    EXIT_REQUESTED.load(Ordering::SeqCst)
}
