use std::process::Command;

fn main() {
    let out = Command::new("du")
        .arg("-sk")
        .arg("/opt/homebrew/Cellar")
        .output()
        .unwrap();
    println!("{:?}", String::from_utf8_lossy(&out.stdout));
}
