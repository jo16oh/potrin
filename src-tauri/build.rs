use std::process::exit;
use std::process::Command;

fn main() {
    let status = Command::new("sh")
        .arg("generate-db.sh")
        .status()
        .expect("failed to execute generate-db.sh");

    if !status.success() {
        eprintln!("generate-db.sh failed with status: {}", status);
        exit(1);
    }

    tauri_build::build()
}
