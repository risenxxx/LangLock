//! Low-level keyboard hook for intercepting Caps Lock and switching input language.

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CAPITAL, VK_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

/// WM_INPUTLANGCHANGEREQUEST message constant.
const WM_INPUTLANGCHANGEREQUEST: u32 = 0x0050;

/// Flag to cycle to the next input language.
const INPUTLANGCHANGE_FORWARD: usize = 0x0002;

/// LLKHF_INJECTED flag value (0x10).
const LLKHF_INJECTED: u32 = 0x00000010;

/// Global hook handle stored as raw pointer for thread safety.
static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

/// Flag to track if we're currently suppressing a Caps Lock press.
static CAPS_LOCK_DOWN: AtomicBool = AtomicBool::new(false);

/// Flag to enable Shift+Caps Lock for regular Caps Lock behavior.
static SHIFT_CAPSLOCK_ENABLED: AtomicBool = AtomicBool::new(false);

/// Checks if the Shift+Caps Lock feature is enabled.
pub fn is_shift_capslock_enabled() -> bool {
    SHIFT_CAPSLOCK_ENABLED.load(Ordering::SeqCst)
}

/// Enables or disables the Shift+Caps Lock feature.
pub fn set_shift_capslock_enabled(enabled: bool) {
    SHIFT_CAPSLOCK_ENABLED.store(enabled, Ordering::SeqCst);
}

/// Installs the low-level keyboard hook.
///
/// # Returns
/// `Ok(())` if the hook was installed successfully, or an error message.
pub fn install_hook() -> Result<(), String> {
    let hook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)
            .map_err(|e| format!("Failed to install keyboard hook: {}", e))?
    };

    HOOK_HANDLE.store(hook.0 as isize, Ordering::SeqCst);
    Ok(())
}

/// Uninstalls the keyboard hook.
pub fn uninstall_hook() {
    let handle = HOOK_HANDLE.swap(0, Ordering::SeqCst);
    if handle != 0 {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(handle as *mut _));
        }
    }
}

/// Switches the input language of the foreground window.
fn switch_language() {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd != HWND::default() {
            // Post WM_INPUTLANGCHANGEREQUEST with INPUTLANGCHANGE_FORWARD to cycle to next language
            let _ = PostMessageW(
                hwnd,
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(INPUTLANGCHANGE_FORWARD),
                LPARAM(0),
            );
        }
    }
}

/// Low-level keyboard hook callback procedure.
///
/// This function intercepts Caps Lock key events and:
/// - Swallows both keydown and keyup events to prevent Caps Lock toggle
/// - Triggers language switch on keydown
/// - Ignores injected keystrokes (from other software)
unsafe extern "system" fn keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code == HC_ACTION as i32 {
        let kb_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode;
        let flags = kb_struct.flags;

        // Check if this is Caps Lock
        if vk_code == VK_CAPITAL.0 as u32 {
            // Ignore injected keystrokes (from other software)
            if (flags.0 & LLKHF_INJECTED) != 0 {
                return CallNextHookEx(None, n_code, w_param, l_param);
            }

            // If Shift+Caps Lock feature is enabled and Shift is held, pass through
            if SHIFT_CAPSLOCK_ENABLED.load(Ordering::SeqCst) {
                let shift_state = GetAsyncKeyState(VK_SHIFT.0 as i32);
                if (shift_state as u16 & 0x8000) != 0 {
                    return CallNextHookEx(None, n_code, w_param, l_param);
                }
            }

            let msg = w_param.0 as u32;

            match msg {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    // Only switch language on initial keydown, not repeats
                    if !CAPS_LOCK_DOWN.swap(true, Ordering::SeqCst) {
                        switch_language();
                    }
                    // Return 1 to swallow the keypress
                    return LRESULT(1);
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    CAPS_LOCK_DOWN.store(false, Ordering::SeqCst);
                    // Return 1 to swallow the key release
                    return LRESULT(1);
                }
                _ => {}
            }
        }
    }

    // Pass through all other keys
    CallNextHookEx(None, n_code, w_param, l_param)
}
