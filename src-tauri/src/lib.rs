// PureVox Tauri 后端入口
// 已注册命令：
//   - bili_search: 调用 B 站官方搜索 API（视频类型），通过 reqwest + cookie_store 绕过浏览器 CORS/反爬
//   - window_minimize / window_maximize / window_close: 自定义窗口控制（decorations: false）

mod bili;
mod llm;
mod rank;
mod storage;

mod window_cmd {
    #[tauri::command]
    pub fn minimize(window: tauri::Window) {
        let _ = window.minimize();
    }

    #[tauri::command]
    pub fn maximize(window: tauri::Window) {
        let _ = window.maximize();
    }

    #[tauri::command]
    pub fn close(window: tauri::Window) {
        let _ = window.close();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            // 预热 B 站 cookie：访问首页拿 buvid3 / b_lsid 等，避免后续搜索被风控
            tauri::async_runtime::spawn(async {
                let _ = bili::warmup_cookies().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bili::bili_search,
            bili::bili_resolve_video,
            bili::ai_filter_tracks,
            bili::fetch_cover,
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
