use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent, Wry,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// Width / height of `cats.webm`.
const ASPECT: f64 = 502.0 / 540.0;
const MAX_GROUPS: usize = 5;
const MIN_H: f64 = 120.0;
const MAX_H: f64 = 900.0;
const DEFAULT_H: f64 = 320.0;
const SIZES: [(f64, &str); 3] = [(200.0, "小"), (320.0, "中"), (520.0, "大")];
const OPACITIES: [(f64, &str); 3] = [(1.0, "100%"), (0.75, "75%"), (0.5, "50%")];
/// WebView2 shares one browser environment per process, so every window
/// must use identical args. This lets the menu unmute without a user gesture.
const BROWSER_ARGS: &str = "--autoplay-policy=no-user-gesture-required";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Group {
    id: u32,
    /// Physical outer position; `None` means "pick a default".
    x: Option<i32>,
    y: Option<i32>,
    /// Logical height in px.
    height: f64,
    opacity: f64,
    locked: bool,
    passthrough: bool,
}

impl Group {
    fn new(id: u32) -> Self {
        Self {
            id,
            x: None,
            y: None,
            height: DEFAULT_H,
            opacity: 1.0,
            locked: false,
            passthrough: false,
        }
    }
    fn label(&self) -> String {
        format!("cat-{}", self.id)
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Saved {
    groups: Vec<Group>,
}

struct AppState {
    groups: Mutex<Vec<Group>>,
    music_owner: Mutex<Option<u32>>,
    hidden: AtomicBool,
    dirty: AtomicBool,
    path: PathBuf,
    tray_add: Mutex<Option<MenuItem<Wry>>>,
    tray_autostart: Mutex<Option<CheckMenuItem<Wry>>>,
}

impl AppState {
    fn group(&self, id: u32) -> Option<Group> {
        self.groups.lock().unwrap().iter().find(|g| g.id == id).cloned()
    }
    fn update(&self, id: u32, f: impl FnOnce(&mut Group)) -> Option<Group> {
        let mut groups = self.groups.lock().unwrap();
        let g = groups.iter_mut().find(|g| g.id == id)?;
        f(g);
        self.dirty.store(true, Ordering::Relaxed);
        Some(g.clone())
    }
    fn save_now(&self) {
        let saved = Saved {
            groups: self.groups.lock().unwrap().clone(),
        };
        if let Some(dir) = self.path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(&saved) {
            let tmp = self.path.with_extension("json.tmp");
            if fs::write(&tmp, json).is_ok() {
                let _ = fs::rename(&tmp, &self.path);
            }
        }
    }
}

fn id_of(label: &str) -> Option<u32> {
    label.strip_prefix("cat-")?.parse().ok()
}

fn load(path: &PathBuf) -> Vec<Group> {
    let groups: Vec<Group> = fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Saved>(&s).ok())
        .map(|s| s.groups)
        .unwrap_or_default();
    let mut groups: Vec<Group> = groups.into_iter().take(MAX_GROUPS).collect();
    for g in &mut groups {
        g.height = g.height.clamp(MIN_H, MAX_H);
        g.opacity = g.opacity.clamp(0.2, 1.0);
    }
    if groups.is_empty() {
        groups.push(Group::new(1));
    }
    groups
}

// ---------------------------------------------------------------- windows

/// True when the window rect (physical) overlaps any monitor enough to grab it.
fn on_some_monitor(app: &AppHandle, x: i32, y: i32, w: u32, h: u32) -> bool {
    let Ok(monitors) = app.available_monitors() else {
        return false;
    };
    monitors.iter().any(|m| {
        let p = m.position();
        let s = m.size();
        let ix = (x + w as i32).min(p.x + s.width as i32) - x.max(p.x);
        let iy = (y + h as i32).min(p.y + s.height as i32) - y.max(p.y);
        ix >= 60 && iy >= 60
    })
}

fn default_position(app: &AppHandle, w: u32, h: u32) -> PhysicalPosition<i32> {
    if let Ok(Some(m)) = app.primary_monitor() {
        let area = m.work_area();
        let margin = (24.0 * m.scale_factor()) as i32;
        return PhysicalPosition::new(
            area.position.x + area.size.width as i32 - w as i32 - margin,
            area.position.y + area.size.height as i32 - h as i32 - margin,
        );
    }
    PhysicalPosition::new(100, 100)
}

fn create_cat_window(app: &AppHandle, g: &Group) -> tauri::Result<WebviewWindow> {
    let win = WebviewWindowBuilder::new(app, g.label(), WebviewUrl::App("index.html".into()))
        .title("three-cat-everything")
        .inner_size(g.height * ASPECT, g.height)
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .focused(false)
        .visible(false)
        .additional_browser_args(BROWSER_ARGS)
        .build()?;

    let size = win.outer_size()?;
    let pos = match (g.x, g.y) {
        (Some(x), Some(y)) if on_some_monitor(app, x, y, size.width, size.height) => {
            PhysicalPosition::new(x, y)
        }
        _ => default_position(app, size.width, size.height),
    };
    win.set_position(pos)?;
    if g.passthrough {
        win.set_ignore_cursor_events(true)?;
    }
    if !app.state::<AppState>().hidden.load(Ordering::Relaxed) {
        win.show()?;
    }
    Ok(win)
}

fn apply_group(app: &AppHandle, g: &Group) {
    if let Some(win) = app.get_webview_window(&g.label()) {
        let _ = win.set_size(LogicalSize::new(g.height * ASPECT, g.height));
        let _ = win.set_ignore_cursor_events(g.passthrough);
    }
    let _ = app.emit("group-changed", g.clone());
}

/// Resize around the window's centre so zooming doesn't make it jump.
fn resize_group(app: &AppHandle, id: u32, height: f64) {
    let height = height.clamp(MIN_H, MAX_H);
    let state = app.state::<AppState>();
    let Some(win) = app.get_webview_window(&format!("cat-{id}")) else {
        return;
    };
    let scale = win.scale_factor().unwrap_or(1.0);
    if let (Ok(pos), Ok(size)) = (win.outer_position(), win.outer_size()) {
        let new_w = (height * ASPECT * scale).round() as i32;
        let new_h = (height * scale).round() as i32;
        let cx = pos.x + size.width as i32 / 2;
        let cy = pos.y + size.height as i32 / 2;
        let _ = win.set_size(LogicalSize::new(height * ASPECT, height));
        let _ = win.set_position(PhysicalPosition::new(cx - new_w / 2, cy - new_h / 2));
    }
    if let Some(g) = state.update(id, |g| g.height = height) {
        let _ = app.emit("group-changed", g);
    }
}

fn add_group(app: &AppHandle, near: Option<u32>) {
    let state = app.state::<AppState>();
    let g = {
        let mut groups = state.groups.lock().unwrap();
        if groups.len() >= MAX_GROUPS {
            return;
        }
        let id = (1..).find(|i| !groups.iter().any(|g| g.id == *i)).unwrap();
        let mut g = Group::new(id);
        let src = near
            .and_then(|n| groups.iter().find(|g| g.id == n))
            .or(groups.last());
        if let Some(src) = src {
            g.height = src.height;
            if let Some(win) = app.get_webview_window(&src.label()) {
                if let (Ok(p), Ok(s)) = (win.outer_position(), win.outer_size()) {
                    // Step left so the new group sits beside its parent.
                    let gap = (40.0 * win.scale_factor().unwrap_or(1.0)) as i32;
                    g.x = Some(p.x - s.width as i32 + gap);
                    g.y = Some(p.y);
                }
            }
        }
        groups.push(g.clone());
        g
    };
    state.dirty.store(true, Ordering::Relaxed);
    let _ = create_cat_window(app, &g);
    refresh_tray(app);
}

fn remove_group(app: &AppHandle, id: u32) {
    let state = app.state::<AppState>();
    {
        let mut groups = state.groups.lock().unwrap();
        if groups.len() <= 1 {
            return;
        }
        groups.retain(|g| g.id != id);
    }
    {
        let mut owner = state.music_owner.lock().unwrap();
        if *owner == Some(id) {
            *owner = None;
        }
    }
    state.dirty.store(true, Ordering::Relaxed);
    if let Some(win) = app.get_webview_window(&format!("cat-{id}")) {
        let _ = win.destroy();
    }
    refresh_tray(app);
}

fn cat_windows(app: &AppHandle) -> Vec<WebviewWindow> {
    app.webview_windows()
        .into_iter()
        .filter(|(label, _)| id_of(label).is_some())
        .map(|(_, w)| w)
        .collect()
}

fn set_hidden(app: &AppHandle, hidden: bool) {
    app.state::<AppState>().hidden.store(hidden, Ordering::Relaxed);
    for w in cat_windows(app) {
        let _ = if hidden { w.hide() } else { w.show() };
    }
}

fn release_all_passthrough(app: &AppHandle) {
    let state = app.state::<AppState>();
    let ids: Vec<u32> = state.groups.lock().unwrap().iter().map(|g| g.id).collect();
    for id in ids {
        if let Some(g) = state.update(id, |g| g.passthrough = false) {
            apply_group(app, &g);
        }
    }
}

fn open_about(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("about") {
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "about", WebviewUrl::App("about.html".into()))
        .title("關於 three-cat-everything")
        .inner_size(380.0, 300.0)
        .resizable(false)
        .maximizable(false)
        .center()
        .additional_browser_args(BROWSER_ARGS)
        .build();
}

fn quit(app: &AppHandle) {
    app.state::<AppState>().save_now();
    app.exit(0);
}

// ---------------------------------------------------------------- menus

fn context_menu(app: &AppHandle, g: &Group) -> tauri::Result<Menu<Wry>> {
    let state = app.state::<AppState>();
    let id = g.id;
    let playing = *state.music_owner.lock().unwrap() == Some(id);
    let count = state.groups.lock().unwrap().len();

    let music = CheckMenuItem::with_id(app, format!("g:{id}:music"), "音樂", true, playing, None::<&str>)?;

    let size_items = SIZES
        .iter()
        .map(|(h, name)| {
            let checked = (g.height - h).abs() < 1.0;
            CheckMenuItem::with_id(app, format!("g:{id}:size:{h}"), *name, true, checked, None::<&str>)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let size_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        size_items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<Wry>).collect();
    let size = Submenu::with_items(app, "大小", true, &size_refs)?;

    let op_items = OPACITIES
        .iter()
        .map(|(o, name)| {
            let checked = (g.opacity - o).abs() < 0.01;
            CheckMenuItem::with_id(app, format!("g:{id}:op:{o}"), *name, true, checked, None::<&str>)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let op_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        op_items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<Wry>).collect();
    let opacity = Submenu::with_items(app, "透明度", true, &op_refs)?;

    let lock = CheckMenuItem::with_id(app, format!("g:{id}:lock"), "鎖定位置", true, g.locked, None::<&str>)?;
    let pass = MenuItem::with_id(
        app,
        format!("g:{id}:pass"),
        "滑鼠穿透（從系統匣解除）",
        true,
        None::<&str>,
    )?;
    let add = MenuItem::with_id(
        app,
        format!("g:{id}:add"),
        format!("再來一組（{count}/{MAX_GROUPS}）"),
        count < MAX_GROUPS,
        None::<&str>,
    )?;
    let remove = MenuItem::with_id(app, format!("g:{id}:remove"), "移除這一組", count > 1, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "關於", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "結束", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    Menu::with_items(
        app,
        &[
            &music, &sep1, &size, &opacity, &lock, &pass, &sep2, &add, &remove, &sep3, &about,
            &quit,
        ],
    )
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let count = state.groups.lock().unwrap().len();
    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);

    let add = MenuItem::with_id(app, "t:add", "再來一組", count < MAX_GROUPS, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "t:toggle", "全部顯示 / 隱藏", true, None::<&str>)?;
    let unpass = MenuItem::with_id(app, "t:unpass", "解除全部滑鼠穿透", true, None::<&str>)?;
    let autostart =
        CheckMenuItem::with_id(app, "t:autostart", "開機自動啟動", true, autostart_on, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "關於", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "結束", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[&add, &toggle, &unpass, &sep1, &autostart, &sep2, &about, &quit],
    )?;

    *state.tray_add.lock().unwrap() = Some(add);
    *state.tray_autostart.lock().unwrap() = Some(autostart);

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().expect("app icon"))
        .tooltip("three-cat-everything")
        .menu(&menu)
        // Left click opens the menu too: when cats are click-through the tray
        // is the only way back, so it should be as easy to reach as possible.
        .show_menu_on_left_click(true)
        .build(app)?;
    Ok(())
}

fn refresh_tray(app: &AppHandle) {
    let state = app.state::<AppState>();
    let count = state.groups.lock().unwrap().len();
    let add = state.tray_add.lock().unwrap().clone();
    if let Some(add) = add {
        let _ = add.set_enabled(count < MAX_GROUPS);
    }
}

fn handle_menu(app: &AppHandle, id: &str) {
    match id {
        "about" => return open_about(app),
        "quit" => return quit(app),
        "t:add" => return add_group(app, None),
        "t:toggle" => {
            let hidden = app.state::<AppState>().hidden.load(Ordering::Relaxed);
            return set_hidden(app, !hidden);
        }
        "t:unpass" => {
            set_hidden(app, false);
            return release_all_passthrough(app);
        }
        "t:autostart" => {
            let al = app.autolaunch();
            let want = !al.is_enabled().unwrap_or(false);
            let _ = if want { al.enable() } else { al.disable() };
            let now = al.is_enabled().unwrap_or(false);
            let item = app.state::<AppState>().tray_autostart.lock().unwrap().clone();
            if let Some(item) = item {
                let _ = item.set_checked(now);
            }
            return;
        }
        _ => {}
    }

    // g:{id}:{action}[:{arg}]
    let mut parts = id.split(':');
    if parts.next() != Some("g") {
        return;
    }
    let Some(gid) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
        return;
    };
    let action = parts.next().unwrap_or("");
    let arg = parts.next().and_then(|s| s.parse::<f64>().ok());
    let state = app.state::<AppState>();

    match action {
        "music" => {
            let _ = app.emit("toggle-music", gid);
        }
        "size" => {
            if let Some(h) = arg {
                resize_group(app, gid, h);
            }
        }
        "op" => {
            if let Some(g) = state.update(gid, |g| g.opacity = arg.unwrap_or(1.0)) {
                apply_group(app, &g);
            }
        }
        "lock" => {
            if let Some(g) = state.update(gid, |g| g.locked = !g.locked) {
                apply_group(app, &g);
            }
        }
        "pass" => {
            if let Some(g) = state.update(gid, |g| g.passthrough = true) {
                apply_group(app, &g);
            }
        }
        "add" => add_group(app, Some(gid)),
        "remove" => remove_group(app, gid),
        _ => {}
    }
}

// ---------------------------------------------------------------- commands

#[tauri::command]
fn get_group(window: WebviewWindow, state: tauri::State<AppState>) -> Option<Group> {
    state.group(id_of(window.label())?)
}

#[tauri::command]
fn set_music(app: AppHandle, window: WebviewWindow, playing: bool) {
    let Some(id) = id_of(window.label()) else {
        return;
    };
    let state = app.state::<AppState>();
    let mut owner = state.music_owner.lock().unwrap();
    if playing {
        *owner = Some(id);
        drop(owner);
        let _ = app.emit("music-owner", id);
    } else if *owner == Some(id) {
        *owner = None;
    }
}

#[tauri::command]
fn scale_group(app: AppHandle, window: WebviewWindow, factor: f64) {
    let Some(id) = id_of(window.label()) else {
        return;
    };
    let Some(g) = app.state::<AppState>().group(id) else {
        return;
    };
    if g.locked {
        return;
    }
    resize_group(&app, id, g.height * factor.clamp(0.5, 2.0));
}

#[tauri::command]
fn show_context_menu(app: AppHandle, window: WebviewWindow) {
    let Some(g) = id_of(window.label()).and_then(|id| app.state::<AppState>().group(id)) else {
        return;
    };
    if let Ok(menu) = context_menu(&app, &g) {
        let _ = window.popup_menu(&menu);
    }
}

// ---------------------------------------------------------------- entry

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            set_hidden(app, false);
            for w in cat_windows(app) {
                let _ = w.set_always_on_top(true);
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![
            get_group,
            set_music,
            scale_group,
            show_context_menu
        ])
        .on_menu_event(|app, event| handle_menu(app, event.id().as_ref()))
        .on_window_event(|window, event| {
            let Some(id) = id_of(window.label()) else {
                return;
            };
            if let WindowEvent::Moved(pos) = event {
                let state = window.app_handle().state::<AppState>();
                state.update(id, |g| {
                    g.x = Some(pos.x);
                    g.y = Some(pos.y);
                });
            }
        })
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("state.json");
            let groups = load(&path);
            app.manage(AppState {
                groups: Mutex::new(groups.clone()),
                music_owner: Mutex::new(None),
                hidden: AtomicBool::new(false),
                dirty: AtomicBool::new(false),
                path,
                tray_add: Mutex::new(None),
                tray_autostart: Mutex::new(None),
            });

            let handle = app.handle().clone();
            for g in &groups {
                create_cat_window(&handle, g)?;
            }
            build_tray(&handle)?;

            // Debounced persistence: write at most every 500ms.
            let saver = handle.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(500));
                let state = saver.state::<AppState>();
                if state.dirty.swap(false, Ordering::Relaxed) {
                    state.save_now();
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building three-cat-everything")
        .run(|app, event| {
            // Closing the about window must not quit the app.
            if let tauri::RunEvent::ExitRequested { code: None, api, .. } = event {
                api.prevent_exit();
            }
            let _ = app;
        });
}
