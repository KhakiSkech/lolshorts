use anyhow::Result;
/// Global hotkey system for LoLShorts
///
/// Registers system-wide hotkeys for recording control:
/// - F8: Toggle auto-capture (start/stop)
/// - F9: Save last 60 seconds (instant replay)
/// - F10: Quick save 30 seconds
///
/// Uses Windows RegisterHotKey API for global hotkey registration
use std::sync::Arc;
use tokio::sync::RwLock;

// UTF-16 string macro for Windows API - MUST be defined before use
#[cfg(target_os = "windows")]
macro_rules! w {
    ($s:expr) => {{
        const STR: &str = concat!($s, "\0");
        const UTF16: &[u16] = &{
            let bytes = STR.as_bytes();
            let mut utf16 = [0u16; STR.len()];
            let mut i = 0;
            while i < bytes.len() {
                utf16[i] = bytes[i] as u16;
                i += 1;
            }
            utf16
        };
        windows::core::PCWSTR::from_raw(UTF16.as_ptr())
    }};
}

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT, VK_F10, VK_F8, VK_F9,
    },
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
        TranslateMessage, CS_HREDRAW, CS_VREDRAW, MSG, WINDOW_EX_STYLE, WM_DESTROY, WM_HOTKEY,
        WNDCLASSW, WS_OVERLAPPEDWINDOW,
    },
};

/// Hotkey identifiers
const HOTKEY_F8: i32 = 1; // Toggle auto-capture
const HOTKEY_F9: i32 = 2; // Save 60s
const HOTKEY_F10: i32 = 3; // Save 30s

/// Hotkey event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    ToggleAutoCapture, // F8
    SaveReplay60,      // F9
    SaveReplay30,      // F10
}

/// Hotkey manager
pub struct HotkeyManager {
    enabled: Arc<RwLock<bool>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Start hotkey listener (Windows implementation)
    #[cfg(target_os = "windows")]
    pub async fn start<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(HotkeyEvent) + Send + Sync + 'static,
    {
        let enabled = Arc::clone(&self.enabled);

        // Mark as enabled
        *enabled.write().await = true;

        // Spawn hotkey listener thread
        tokio::task::spawn_blocking(move || {
            unsafe {
                // Create invisible window for message processing
                let h_instance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                    .expect("Failed to get module handle");

                let class_name = w!("LoLShortsHotkeyWindow");

                let wc = WNDCLASSW {
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(window_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: h_instance.into(),
                    hIcon: Default::default(),
                    hCursor: Default::default(),
                    hbrBackground: Default::default(),
                    lpszMenuName: windows::core::PCWSTR::null(),
                    lpszClassName: class_name,
                };

                windows::Win32::UI::WindowsAndMessaging::RegisterClassW(&wc);

                let hwnd = match CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class_name,
                    w!("LoLShorts Hotkeys"),
                    WS_OVERLAPPEDWINDOW,
                    0,
                    0,
                    0,
                    0,
                    HWND::default(),
                    None,
                    h_instance,
                    None,
                ) {
                    Ok(hwnd) => hwnd,
                    Err(e) => {
                        tracing::error!("Failed to create hotkey window: {:?}", e);
                        return;
                    }
                };

                // Register hotkeys
                // F8: Toggle auto-capture (no modifiers)
                if RegisterHotKey(hwnd, HOTKEY_F8, MOD_NOREPEAT, VK_F8.0 as u32).is_err() {
                    tracing::warn!("Failed to register F8 hotkey");
                }

                // F9: Save 60s (no modifiers)
                if RegisterHotKey(hwnd, HOTKEY_F9, MOD_NOREPEAT, VK_F9.0 as u32).is_err() {
                    tracing::warn!("Failed to register F9 hotkey");
                }

                // F10: Save 30s (no modifiers)
                if RegisterHotKey(hwnd, HOTKEY_F10, MOD_NOREPEAT, VK_F10.0 as u32).is_err() {
                    tracing::warn!("Failed to register F10 hotkey");
                }

                tracing::info!(
                    "Global hotkeys registered: F8 (toggle), F9 (save 60s), F10 (save 30s)"
                );

                // Message loop
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                    if msg.message == WM_HOTKEY {
                        let hotkey_id = msg.wParam.0 as i32;
                        let event = match hotkey_id {
                            HOTKEY_F8 => Some(HotkeyEvent::ToggleAutoCapture),
                            HOTKEY_F9 => Some(HotkeyEvent::SaveReplay60),
                            HOTKEY_F10 => Some(HotkeyEvent::SaveReplay30),
                            _ => None,
                        };

                        if let Some(event) = event {
                            tracing::debug!("Hotkey triggered: {:?}", event);
                            callback(event);
                        }
                    }

                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // Cleanup
                UnregisterHotKey(hwnd, HOTKEY_F8).ok();
                UnregisterHotKey(hwnd, HOTKEY_F9).ok();
                UnregisterHotKey(hwnd, HOTKEY_F10).ok();
            }
        });

        Ok(())
    }

    /// Stop hotkey listener
    pub async fn stop(&self) -> Result<()> {
        let mut enabled = self.enabled.write().await;
        *enabled = false;

        // Note: We can't easily send WM_QUIT to the specific window without storing HWND
        // The hotkey thread will exit when the application closes
        // For a cleaner shutdown, we'd need to use a different approach (e.g., a channel)

        tracing::info!("Hotkey manager stopped");

        Ok(())
    }

    /// Check if hotkeys are enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
}

/// Window procedure for hotkey message handling
#[cfg(target_os = "windows")]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// For non-Windows platforms
#[cfg(not(target_os = "windows"))]
impl HotkeyManager {
    pub async fn start<F>(&self, _callback: F) -> Result<()>
    where
        F: Fn(HotkeyEvent) + Send + Sync + 'static,
    {
        tracing::warn!("Global hotkeys not supported on this platform");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hotkey_manager_creation() {
        let manager = HotkeyManager::new();
        assert!(!manager.is_enabled().await);
    }

    #[tokio::test]
    async fn test_hotkey_event_equality() {
        assert_eq!(
            HotkeyEvent::ToggleAutoCapture,
            HotkeyEvent::ToggleAutoCapture
        );
        assert_ne!(HotkeyEvent::ToggleAutoCapture, HotkeyEvent::SaveReplay60);
    }
}
