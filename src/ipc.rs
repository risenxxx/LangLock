//! Single instance enforcement and inter-process communication.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE,
    SYNCHRONIZATION_SYNCHRONIZE,
};

/// Named mutex for single instance enforcement.
const MUTEX_NAME: &str = "Global\\LangLock_SingleInstance";

/// Named event for signaling tray icon restoration.
const EVENT_NAME: &str = "Global\\LangLock_ShowTray";

/// Handle wrapper that closes on drop.
pub struct SafeHandle(HANDLE);

impl Drop for SafeHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Result of attempting to acquire single instance lock.
pub enum SingleInstanceResult {
    /// Successfully acquired the lock - this is the first instance.
    Acquired(SafeHandle),
    /// Another instance is already running.
    AlreadyRunning,
}

/// Attempts to acquire the single instance mutex.
///
/// # Returns
/// - `SingleInstanceResult::Acquired` if this is the first instance
/// - `SingleInstanceResult::AlreadyRunning` if another instance exists
pub fn acquire_single_instance() -> SingleInstanceResult {
    let mutex_name: Vec<u16> = MUTEX_NAME.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let handle = CreateMutexW(None, true, PCWSTR(mutex_name.as_ptr()));

        match handle {
            Ok(h) => {
                // Check if we actually own the mutex or it already existed
                let last_error = windows::Win32::Foundation::GetLastError();
                if last_error.0 == 183 {
                    // ERROR_ALREADY_EXISTS
                    // Another instance owns it
                    let _ = CloseHandle(h);
                    SingleInstanceResult::AlreadyRunning
                } else {
                    SingleInstanceResult::Acquired(SafeHandle(h))
                }
            }
            Err(_) => SingleInstanceResult::AlreadyRunning,
        }
    }
}

/// Creates or opens the show-tray event.
///
/// # Returns
/// A handle to the event, or None if creation failed.
pub fn create_show_event() -> Option<SafeHandle> {
    let event_name: Vec<u16> = EVENT_NAME.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        // Create an auto-reset event (resets after a single wait is satisfied)
        let handle: Result<HANDLE, _> =
            CreateEventW(None, false, false, PCWSTR(event_name.as_ptr()));

        match handle {
            Ok(h) if !h.is_invalid() => Some(SafeHandle(h)),
            _ => None,
        }
    }
}

/// Signals the show-tray event to notify the running instance.
///
/// Called by the second instance to tell the first instance to restore the tray icon.
pub fn signal_show_tray() {
    let event_name: Vec<u16> = EVENT_NAME.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        // Open the existing event
        if let Ok(handle) = OpenEventW(
            EVENT_MODIFY_STATE | SYNCHRONIZATION_SYNCHRONIZE,
            false,
            PCWSTR(event_name.as_ptr()),
        ) {
            if !handle.is_invalid() {
                let _ = SetEvent(handle);
                let _ = CloseHandle(handle);
            }
        }
    }
}

/// Checks if the show-tray event has been signaled.
///
/// # Arguments
/// * `event` - The event handle to check.
///
/// # Returns
/// `true` if the event was signaled, `false` otherwise.
pub fn check_show_signal(event: &SafeHandle) -> bool {
    unsafe {
        // Non-blocking check (0 timeout)
        let result = WaitForSingleObject(event.0, 0);
        result == WAIT_OBJECT_0
    }
}
