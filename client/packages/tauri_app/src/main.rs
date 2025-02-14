use tauri::{command, generate_handler, Builder};

#[command]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    Builder::default()
        .invoke_handler(generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
