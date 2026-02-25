//! LangLock - A lightweight Windows utility that intercepts Caps Lock to switch input language.
//!
//! Unlike solutions that emulate virtual keystrokes, LangLock sends `WM_INPUTLANGCHANGEREQUEST`
//! directly to the foreground window, making it safe for fast touch-typing and invisible to
//! anti-cheat systems.

#![windows_subsystem = "windows"]

mod hook;
mod ipc;
mod notification;
mod startup;
mod tray;

use ipc::{acquire_single_instance, check_show_signal, create_show_event, signal_show_tray, SingleInstanceResult};
use muda::MenuEvent;
use std::time::Duration;
use tray::{handle_menu_event, is_exit_requested, TrayManager};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
};

fn main() {
    // Attempt to acquire single instance lock
    let _mutex_handle = match acquire_single_instance() {
        SingleInstanceResult::Acquired(handle) => handle,
        SingleInstanceResult::AlreadyRunning => {
            // Another instance is running - signal it to show tray and exit
            signal_show_tray();
            return;
        }
    };

    // Create the show-tray event for IPC
    let show_event = create_show_event();

    // Install the keyboard hook
    if let Err(e) = hook::install_hook() {
        eprintln!("Failed to install keyboard hook: {}", e);
        return;
    }

    // Create the system tray
    let mut tray_manager = match TrayManager::new() {
        Ok(tray) => tray,
        Err(e) => {
            eprintln!("Failed to create tray icon: {}", e);
            hook::uninstall_hook();
            return;
        }
    };

    // Main message loop
    run_message_loop(&mut tray_manager, show_event.as_ref());

    // Cleanup
    hook::uninstall_hook();
}

/// Runs the Windows message loop with menu event handling and IPC checking.
fn run_message_loop(tray: &mut TrayManager, show_event: Option<&ipc::SafeHandle>) {
    let menu_receiver = MenuEvent::receiver();

    loop {
        // Check for exit request
        if is_exit_requested() {
            break;
        }

        // Process Windows messages (non-blocking)
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        // Handle menu events (non-blocking)
        if let Ok(event) = menu_receiver.try_recv() {
            if handle_menu_event(event, tray) {
                break;
            }
        }

        // Check for show-tray signal from another instance
        if let Some(event) = show_event {
            if check_show_signal(event) {
                if let Err(e) = tray.show() {
                    eprintln!("Failed to show tray: {}", e);
                }
            }
        }

        // Small sleep to prevent busy-waiting
        std::thread::sleep(Duration::from_millis(10));
    }
}
