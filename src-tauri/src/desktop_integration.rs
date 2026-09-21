use std::sync::Mutex;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub enum MacosServiceAction {
    AddVocabulary,
    Translate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacosServiceRequest {
    pub action: MacosServiceAction,
    pub text: String,
}

static PENDING_REQUESTS: Mutex<Vec<MacosServiceRequest>> = Mutex::new(Vec::new());

pub fn take_pending_requests() -> Vec<MacosServiceRequest> {
    PENDING_REQUESTS
        .lock()
        .map(|mut requests| std::mem::take(&mut *requests))
        .unwrap_or_default()
}

pub fn setup(_app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    macos::setup(_app)?;
    Ok(())
}

pub fn handle_window_event(_window: &tauri::Window, _event: &tauri::WindowEvent) {
    #[cfg(target_os = "macos")]
    if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
        api.prevent_close();
        let _ = _window.hide();
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::sync::OnceLock;

    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{ClassType, MainThreadOnly, define_class, msg_send};
    use objc2_app_kit::{
        NSApplication, NSPasteboard, NSPasteboardTypeString, NSUpdateDynamicServices,
    };
    use objc2_foundation::{MainThreadMarker, NSArray, NSObject, NSObjectProtocol, NSString};
    use tauri::menu::MenuBuilder;
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::{AppHandle, Emitter, Manager};

    use super::{MacosServiceAction, MacosServiceRequest, PENDING_REQUESTS};

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

    #[derive(Debug, Default)]
    struct ServiceProviderIvars;

    define_class!(
        #[unsafe(super = NSObject)]
        #[thread_kind = MainThreadOnly]
        #[ivars = ServiceProviderIvars]
        struct ServiceProvider;

        unsafe impl NSObjectProtocol for ServiceProvider {}

        impl ServiceProvider {
            #[unsafe(method(translateSelection:userData:error:))]
            fn translate_selection(
                &self,
                pasteboard: &NSPasteboard,
                _user_data: Option<&NSString>,
                _error: *mut *mut NSString,
            ) {
                queue_selection(MacosServiceAction::Translate, pasteboard);
            }

            #[unsafe(method(addSelectionToVocabulary:userData:error:))]
            fn add_selection_to_vocabulary(
                &self,
                pasteboard: &NSPasteboard,
                _user_data: Option<&NSString>,
                _error: *mut *mut NSString,
            ) {
                queue_selection(MacosServiceAction::AddVocabulary, pasteboard);
            }
        }
    );

    impl ServiceProvider {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(ServiceProviderIvars);
            unsafe { msg_send![super(this), init] }
        }
    }

    pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
        let _ = APP_HANDLE.set(app.handle().clone());
        setup_tray(app)?;
        setup_services()?;
        Ok(())
    }

    fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
        let menu = MenuBuilder::new(app)
            .text("show", "Open Polarbear Vocab")
            .separator()
            .text("quit", "Quit Polarbear Vocab")
            .build()?;
        let icon = app
            .default_window_icon()
            .cloned()
            .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".to_owned()))?;
        TrayIconBuilder::with_id("polarbear-vocab")
            .icon(icon)
            .icon_as_template(false)
            .tooltip("Polarbear Vocab")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| match event.id().as_ref() {
                "show" => show_main_window(app),
                "quit" => app.exit(0),
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if matches!(
                    event,
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                ) {
                    show_main_window(tray.app_handle());
                }
            })
            .build(app)?;
        Ok(())
    }

    fn setup_services() -> Result<(), Box<dyn std::error::Error>> {
        let mtm = MainThreadMarker::new().ok_or("macOS services must initialize on main thread")?;
        let application = NSApplication::sharedApplication(mtm);
        let provider = ServiceProvider::new(mtm);
        let provider = Retained::into_raw(provider);
        let provider: &AnyObject = unsafe { (&*provider).as_super().as_super() };
        unsafe { application.setServicesProvider(Some(provider)) };
        let send_types = NSArray::from_slice(&[unsafe { NSPasteboardTypeString }]);
        let return_types = NSArray::from_slice(&[]);
        application.registerServicesMenuSendTypes_returnTypes(&send_types, &return_types);
        NSUpdateDynamicServices();
        Ok(())
    }

    fn queue_selection(action: MacosServiceAction, pasteboard: &NSPasteboard) {
        let Some(value) = pasteboard.stringForType(unsafe { NSPasteboardTypeString }) else {
            return;
        };
        let text = truncate_selection(value.to_string().trim(), 20_000);
        if text.is_empty() {
            return;
        }
        if let Ok(mut requests) = PENDING_REQUESTS.lock() {
            requests.push(MacosServiceRequest { action, text });
        }
        if let Some(app) = APP_HANDLE.get() {
            show_main_window(app);
            let _ = app.emit("macos-service-pending", ());
        }
    }

    fn truncate_selection(value: &str, maximum: usize) -> String {
        value.chars().take(maximum).collect()
    }

    fn show_main_window(app: &AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MacosServiceAction, MacosServiceRequest, PENDING_REQUESTS, take_pending_requests};

    #[test]
    fn pending_service_requests_are_drained_once() {
        PENDING_REQUESTS.lock().unwrap().push(MacosServiceRequest {
            action: MacosServiceAction::Translate,
            text: "Hello".to_owned(),
        });

        assert_eq!(take_pending_requests().len(), 1);
        assert!(take_pending_requests().is_empty());
    }
}
