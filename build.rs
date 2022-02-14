use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=dictionaries/english/1.txt");
    Command::new("sh")
        .args(&["-c", "./build-dictionaries.sh"])
        .status()
        .unwrap();
}
