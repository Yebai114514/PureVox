mod bili;
mod llm;
mod rank;
mod storage;

use tauri::Manager;

mod window_cmd {
    #[tauri::command]
    pub fn minimize(window: tauri::Window) {
        let _ = window.minimize();
    }

    #[tauri::command]
    pub fn maximize(window: tauri::Window) {
        if window.is_maximized().unwrap_or(false) {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }

    #[tauri::command]
    pub fn close(window: tauri::Window) {
        let _ = window.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};

            // 托盘菜单
            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let icon = app.default_window_icon().cloned().unwrap();
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 预热 B 站 cookie：访问首页拿 buvid3 / b_lsid 等，避免后续搜索被风控
            tauri::async_runtime::spawn(async {
                let _ = bili::warmup_cookies().await;
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            bili::bili_search,
            bili::bili_resolve_video,
            bili::ai_filter_tracks,
            bili::fetch_cover,
            bili::fetch_covers_batch,
            bili::generate_recommend,
            rank::rank_candidates_cmd,
            storage::load_data_file,
            storage::save_data_file,
            storage::load_settings,
            storage::save_settings,
            window_cmd::minimize,
            window_cmd::maximize,
            window_cmd::close
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
