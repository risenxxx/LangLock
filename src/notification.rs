//! Windows toast notification functionality.

use winrt_notification::{Duration, Toast};

/// Application ID for toast notifications.
const APP_ID: &str = "LangLock";

/// Shows a notification when the tray icon is hidden.
///
/// Informs the user that LangLock is still running in the background
/// and how to restore the tray icon.
pub fn show_hidden_notification() {
    let _ = Toast::new(APP_ID)
        .title("LangLock")
        .text1("LangLock is running in the background.")
        .text2("Relaunch the app to restore the tray icon.")
        .duration(Duration::Short)
        .show();
}
