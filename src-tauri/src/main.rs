// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    Router,
    routing::get,
};
use tower_http::{
    services::ServeDir,
    cors::CorsLayer,
};
use std::net::SocketAddr;
use tauri::Manager;

// Tauri commands that can be called from frontend
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to TabAgent Desktop!", name)
}

#[tokio::main]
async fn main() {
    // Start embedded HTTP server in background
    tokio::spawn(async {
        start_embedded_server().await;
    });

    // Start Tauri app
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // Get main window
            let window = app.get_webview_window("main").unwrap();
            
            // Load dashboard on startup
            window.eval("window.location.href = 'http://localhost:3000'").ok();
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn start_embedded_server() {
    println!("🚀 Starting TabAgent embedded server...");

    // Build the router with both UIs
    let app = Router::new()
        // Serve Dashboard (React) at root
        .nest_service("/", ServeDir::new("../dashboard/dist"))
        // Serve Agent Builder (Vue) at /workflows
        .nest_service("/workflows", ServeDir::new("../agent-builder/dist"))
        // API routes (placeholder - integrate your Rust server API)
        .route("/api/health", get(health_check))
        // CORS for development
        .layer(CorsLayer::permissive());

    // Bind to localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("✅ Server listening on http://{}", addr);
    println!("📊 Dashboard:      http://localhost:3000/");
    println!("🤖 Agent Builder:  http://localhost:3000/workflows");

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

