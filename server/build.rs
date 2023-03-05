use std::{
    io::{self, Write},
    process::Command,
};

fn main() {
    let current = std::env::current_dir().unwrap();
    let build_script = current
        .parent()
        .unwrap()
        .join("player-manager")
        .join("build.sh");
    println!("build_script: {:?}", build_script);
    let ab = std::fs::canonicalize(build_script).unwrap();
    let path = ab.to_str().unwrap();

    match Command::new("bash")
        .current_dir("../player-manager")
        .arg("-C")
        .arg(path)
        .output()
    {
        Ok(output) => {
            println!("status: {}", output.status);
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
        Err(e) => {
            println!("error: {}", e);
        }
    };
}
