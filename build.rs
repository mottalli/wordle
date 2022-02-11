use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=1.txt");
    Command::new("sh").args(&["-c", "./build-dictionary.py"]).status().unwrap();
}