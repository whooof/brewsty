use std::process::Command;
fn main() {
    let out = Command::new("sh").arg("-c").arg("du -sk /opt/homebrew/Cellar/* /opt/homebrew/Caskroom/*").output().unwrap();
    println!("{:?}", String::from_utf8_lossy(&out.stdout).lines().take(5).collect::<Vec<_>>());
}
