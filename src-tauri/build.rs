// Bakes GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET from the project-root .env into the
// binary so both `tauri dev` and release builds pick them up.
fn main() {
    println!("cargo:rerun-if-changed=../.env");
    if let Ok(contents) = std::fs::read_to_string("../.env") {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                if key == "GOOGLE_CLIENT_ID" || key == "GOOGLE_CLIENT_SECRET" {
                    let value = value.trim().trim_matches('"');
                    println!("cargo:rustc-env={key}={value}");
                }
            }
        }
    }
    tauri_build::build()
}
