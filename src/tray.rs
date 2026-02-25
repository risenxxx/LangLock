//! System tray icon and menu management.

use crate::notification::show_hidden_notification;
use crate::startup;
use muda::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Menu item IDs.
const MENU_STARTUP_ID: &str = "startup";
const MENU_HIDE_ID: &str = "hide";
const MENU_EXIT_ID: &str = "exit";

/// Global flag indicating if exit was requested.
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Manages the system tray icon and its context menu.
pub struct TrayManager {
    tray_icon: Option<TrayIcon>,
    startup_item: CheckMenuItem,
    hide_item: MenuItem,
    exit_item: MenuItem,
}

impl TrayManager {
    /// Creates a new TrayManager with an icon and context menu.
    pub fn new() -> Result<Self, String> {
        // Check current startup state
        let startup_enabled = startup::is_startup_enabled();

        // Create menu items
        let startup_item = CheckMenuItem::with_id(
            MENU_STARTUP_ID,
            "Run on startup",
            true,
            startup_enabled,
            None,
        );
        let hide_item = MenuItem::with_id(MENU_HIDE_ID, "Hide tray icon", true, None);
        let exit_item = MenuItem::with_id(MENU_EXIT_ID, "Exit", true, None);

        // Build the menu
        let menu = Menu::new();
        menu.append(&startup_item)
            .map_err(|e| format!("Failed to add startup item: {}", e))?;
        menu.append(&hide_item)
            .map_err(|e| format!("Failed to add hide item: {}", e))?;
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| format!("Failed to add separator: {}", e))?;
        menu.append(&exit_item)
            .map_err(|e| format!("Failed to add exit item: {}", e))?;

        // Create the tray icon
        let icon = create_icon()?;
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("LangLock - Caps Lock Language Switcher")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .map_err(|e| format!("Failed to create tray icon: {}", e))?;

        Ok(Self {
            tray_icon: Some(tray_icon),
            startup_item,
            hide_item,
            exit_item,
        })
    }

    /// Hides the tray icon.
    pub fn hide(&mut self) {
        if let Some(tray) = self.tray_icon.take() {
            drop(tray);
            show_hidden_notification();
        }
    }

    /// Shows/restores the tray icon.
    pub fn show(&mut self) -> Result<(), String> {
        if self.tray_icon.is_none() {
            // Update startup state before showing
            let startup_enabled = startup::is_startup_enabled();
            self.startup_item.set_checked(startup_enabled);

            // Build the menu with existing items
            let menu = Menu::new();
            menu.append(&self.startup_item)
                .map_err(|e| format!("Failed to add startup item: {}", e))?;
            menu.append(&self.hide_item)
                .map_err(|e| format!("Failed to add hide item: {}", e))?;
            menu.append(&PredefinedMenuItem::separator())
                .map_err(|e| format!("Failed to add separator: {}", e))?;
            menu.append(&self.exit_item)
                .map_err(|e| format!("Failed to add exit item: {}", e))?;

            // Recreate the tray icon
            let icon = create_icon()?;
            let tray_icon = TrayIconBuilder::new()
                .with_tooltip("LangLock - Caps Lock Language Switcher")
                .with_icon(icon)
                .with_menu(Box::new(menu))
                .build()
                .map_err(|e| format!("Failed to recreate tray icon: {}", e))?;

            self.tray_icon = Some(tray_icon);
        }
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

    false
}

/// Checks if exit has been requested.
pub fn is_exit_requested() -> bool {
    EXIT_REQUESTED.load(Ordering::SeqCst)
}
