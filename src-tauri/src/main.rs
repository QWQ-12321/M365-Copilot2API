mod tray;

use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;

use log::{error, info, warn};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};
use tauri_plugin_shell::ShellExt;

/// 默认监听端口
const DEFAULT_PORT: u16 = 4141;

/// 全局端口状态（供命令读取）
struct PortState(Arc<AtomicU16>);

/// 全局运行标志
struct ServerRunning(Arc<AtomicBool>);

/// 启动 Go sidecar 进程
fn spawn_sidecar(app: &AppHandle, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let binary_name = format!(
        "m365-copilot2api{}",
        if cfg!(windows) { ".exe" } else { "" }
    );

    let (mut rx, _child) = app
        .shell()
        .sidecar(&binary_name)?
        .env("M365_LISTEN", format!("127.0.0.1:{}", port))
        .env("M365_DATA_DIR", data_dir()?)
        .spawn()?;

    // 读取 sidecar 输出到日志
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Stdout(bytes) = event {
                if let Ok(text) = String::from_utf8(bytes) {
                    for line in text.lines() {
                        info!(target: "sidecar", "{}", line);
                    }
                }
            } else if let tauri_plugin_shell::process::CommandEvent::Stderr(bytes) = event {
                if let Ok(text) = String::from_utf8(bytes) {
                    for line in text.lines() {
                        warn!(target: "sidecar", "{}", line);
                    }
                }
            }
        }
    });

    Ok(())
}

/// 计算数据目录
fn data_dir() -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("M365-Copilot2API");
            std::fs::create_dir_all(&dir)?;
            return Ok(dir.to_string_lossy().to_string());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("m365-copilot2api");
            std::fs::create_dir_all(&dir)?;
            return Ok(dir.to_string_lossy().to_string());
        }
    }
    Ok("./data".to_string())
}

/// 等待服务器就绪（轮询端口）
async fn wait_for_server(port: u16, timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        let client = reqwest::Client::new();
        if client
            .get(format!("http://127.0.0.1:{}/api/stats", port))
            .send()
            .await
            .is_ok()
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    false
}

#[tauri::command]
fn get_port(state: State<PortState>) -> u16 {
    state.0.load(Ordering::Relaxed)
}

#[tauri::command]
fn is_server_running(state: State<ServerRunning>) -> bool {
    state.0.load(Ordering::Relaxed)
}

#[tauri::command]
fn open_in_browser(port: u16) {
    let url = format!("http://127.0.0.1:{}/", port);
    if let Err(e) = open::that(&url) {
        error!("无法打开浏览器: {}", e);
    }
}

fn main() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let port = Arc::new(AtomicU16::new(DEFAULT_PORT));
    let running = Arc::new(AtomicBool::new(false));

    // 尝试绑定端口，如果被占用则递增
    let mut try_port = DEFAULT_PORT;
    while try_port < DEFAULT_PORT + 100 {
        if std::net::TcpListener::bind(format!("127.0.0.1:{}", try_port)).is_ok() {
            port.store(try_port, Ordering::Relaxed);
            break;
        }
        try_port += 1;
    }
    let actual_port = port.load(Ordering::Relaxed);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(PortState(port.clone()))
        .manage(ServerRunning(running.clone()))
        .setup(move |app| {
            // 启动 Go sidecar
            if let Err(e) = spawn_sidecar(&app.handle(), actual_port) {
                error!("启动 sidecar 失败: {}", e);
                // 继续启动窗口，让用户看到错误
            } else {
                running.store(true, Ordering::Relaxed);
                info!("M365 Copilot2API 已在端口 {} 启动", actual_port);
            }

            // 系统托盘
            tray::setup_tray(&app.handle(), actual_port)?;

            // 等待服务器就绪后打开窗口
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let ready = rt.block_on(wait_for_server(actual_port, Duration::from_secs(15)));
                if ready {
                    let _ = app_handle.emit("server-ready", ());
                } else {
                    let _ = app_handle.emit("server-error", "服务器启动超时");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_port,
            is_server_running,
            open_in_browser
        ])
        .build(tauri::generate_context!())
        .expect("构建 Tauri 应用失败");

    // 运行事件循环
    app.run(|_app_handle, event| {
        if let RunEvent::Exit = event {
            info!("应用程序退出");
        }
    });
}
