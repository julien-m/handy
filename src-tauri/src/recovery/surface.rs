//! Tauri adapter of the recovery surface: the dedicated recovery window, created on first use
//! and driven on the main thread, sized to what its view reports and placed where the indicator
//! is. The full transcript never leaves the native side. The projection is emitted with
//! `emit_to(RECOVERY_WINDOW, …)`; in Tauri 2 a webview listener created with the default
//! target (`Any`) would still receive it, so the isolation also rests on a convention checked
//! by `tests/recovery_native.rs`: only the recovery window listens to its events. The
//! label-checked `recovery_bubble_view` command remains the source of truth.
use super::platform::{self, INITIAL_SIZE, RECOVERY_WINDOW};
use super::ports::RecoverySurfacePort;
use super::service::RecoveryService;
use super::types::{BubbleId, RecoveryBubbleView};
use std::sync::{Arc, Mutex, PoisonError};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

/// Event carrying the projection to the recovery window.
pub const VIEW_EVENT: &str = "recovery-bubble-changed";
/// Event telling the recovery window its bubble is gone.
pub const HIDDEN_EVENT: &str = "recovery-bubble-hidden";
/// Largest size the view may ask for (logical points).
const MAX_SIZE: (f64, f64) = (560.0, 420.0);

/// Last size reported by the view (the next presentation reuses it).
static SIZE: Mutex<(f64, f64)> = Mutex::new(INITIAL_SIZE);

/// The window as the main thread sees it.
pub struct TauriSurface {
    app: AppHandle,
}
impl TauriSurface {
    /// The surface of `app`.
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
    /// Run `work` on the main thread; `unscheduled` runs here when that is impossible.
    fn on_main(&self, work: impl FnOnce(&AppHandle) + Send + 'static, unscheduled: impl FnOnce()) {
        let handle = self.app.clone();
        if let Err(error) = self.app.run_on_main_thread(move || work(&handle)) {
            log::error!("Recovery surface: main thread unavailable ({error})");
            unscheduled();
        }
    }
}

fn service(app: &AppHandle) -> Option<Arc<RecoveryService>> {
    app.try_state::<Arc<RecoveryService>>()
        .map(|state| Arc::clone(state.inner()))
}
fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(RECOVERY_WINDOW)
}
fn emit_view(app: &AppHandle, view: &RecoveryBubbleView) {
    if let Err(error) = app.emit_to(RECOVERY_WINDOW, VIEW_EVENT, view) {
        log::warn!("Could not update the recovery bubble: {error}");
    }
}
fn failed(app: &AppHandle, bubble_id: BubbleId) {
    if let Some(service) = service(app) {
        service.presentation_failed(bubble_id);
    }
}

/// A notice started holding the indicator, or the held indicator changed its extent: a bubble
/// already shown moves beside it so that neither covers the other. Without a recovery window
/// (gate closed) nothing happens.
pub fn follow_indicator(app: &AppHandle) {
    if let Some(window) = window(app).filter(|window| window.is_visible().unwrap_or(false)) {
        place(app, &window);
    }
}

/// Size and place the window at the indicator's place (beside it while a notice holds it).
fn place(app: &AppHandle, window: &WebviewWindow) {
    let (width, height) = *SIZE.lock().unwrap_or_else(PoisonError::into_inner);
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
    if let Some((x, y)) = crate::overlay::indicator_place(app, width, height) {
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
    }
}

// @spec FR-002: Present the bubble in place of the indicator — .specs/features/010-recovery-bubble/spec.md#fr-002
fn present_on_main(app: &AppHandle, view: &RecoveryBubbleView) {
    crate::overlay::yield_to_recovery(app);
    let window = match window(app).map_or_else(|| platform::create_window(app), Ok) {
        Ok(window) => window,
        Err(error) => {
            log::error!("Recovery window could not be created: {error}");
            return failed(app, view.bubble_id);
        }
    };
    place(app, &window);
    if let Err(error) = platform::show(&window) {
        log::error!("Recovery window could not be shown: {error}");
        return failed(app, view.bubble_id);
    }
    emit_view(app, view);
}

fn hide_on_main(app: &AppHandle) {
    if let Some(window) = window(app) {
        platform::hide(&window);
    }
    let _ = app.emit_to(RECOVERY_WINDOW, HIDDEN_EVENT, ());
}

fn drag_on_main(app: &AppHandle, bubble_id: BubbleId, transcript: &str) {
    let Some(service) = service(app) else {
        return;
    };
    let Some(window) = window(app) else {
        return service.drag_failed(bubble_id);
    };
    let handle = app.clone();
    let ended = Box::new(move |bubble, result| {
        if let Some(service) = self::service(&handle) {
            service.drag_ended(bubble, result);
        }
    });
    if let Err(error) = platform::begin_drag(&window, bubble_id, transcript, ended) {
        log::warn!("Recovery drag not started: {error}");
        service.drag_failed(bubble_id);
    }
}

impl RecoverySurfacePort for TauriSurface {
    fn present(&self, view: RecoveryBubbleView) {
        let (app, bubble_id) = (self.app.clone(), view.bubble_id);
        self.on_main(
            move |app| present_on_main(app, &view),
            move || failed(&app, bubble_id),
        );
    }
    fn update(&self, view: RecoveryBubbleView) {
        emit_view(&self.app, &view);
    }
    fn hide(&self) {
        self.on_main(hide_on_main, || {});
    }
    fn destroy(&self, bubble_id: BubbleId) {
        let app = self.app.clone();
        let work = move |app: &AppHandle| {
            if let Some(window) = window(app) {
                if let Err(error) = window.destroy() {
                    log::error!("Recovery window could not be destroyed: {error}");
                }
            }
            if let Some(service) = service(app) {
                service.window_destroyed(bubble_id);
            }
        };
        self.on_main(work, move || {
            if let Some(service) = service(&app) {
                service.window_destroyed(bubble_id);
            }
        });
    }
    fn begin_drag(&self, bubble_id: BubbleId, transcript: String) {
        let app = self.app.clone();
        self.on_main(
            move |app| drag_on_main(app, bubble_id, &transcript),
            move || {
                if let Some(service) = service(&app) {
                    service.drag_failed(bubble_id);
                }
            },
        );
    }
    fn show_again(&self, view: RecoveryBubbleView) {
        let work = move |app: &AppHandle| {
            if let Some(window) = window(app) {
                // Placed again first: a notice may have started holding during the drag.
                place(app, &window);
                platform::restore_after_drag(&window);
            }
            emit_view(app, &view);
        };
        self.on_main(work, || {});
    }
}

/// The view asked for `width` × `height` (logical points): resize and place the window.
pub fn resize(app: &AppHandle, width: f64, height: f64) {
    if !(width.is_finite() && height.is_finite()) {
        return;
    }
    let size = (width.clamp(1.0, MAX_SIZE.0), height.clamp(1.0, MAX_SIZE.1));
    *SIZE.lock().unwrap_or_else(PoisonError::into_inner) = size;
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = window(&handle) {
            place(&handle, &window);
        }
    });
}
