use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// Generate a synthetic Rust project with many small files to benchmark build speed.
// Usage:
//   cargo run --example generate_synthetic -- <root> <files>
// Example:
//   cargo run --example generate_synthetic -- /tmp/kr_synth 10000
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <root> <files>", args.get(0).map(String::as_str).unwrap_or("generate_synthetic"));
        std::process::exit(2);
    }
    let root = PathBuf::from(&args[1]);
    let n: usize = args[2].parse().expect("files must be a number");

    let src = root.join("src");
    fs::create_dir_all(&src).expect("create src");

    // Write lib.rs that mods chunks to keep module tree shallow-ish
    let mut lib = String::new();
    lib.push_str("// synthetic project generated for benchmarking\n");
    lib.push_str("pub fn root() {}\n");

    for i in 0..n {
        lib.push_str(&format!("pub mod f{};\n", i));
    }
    fs::write(src.join("lib.rs"), lib).expect("write lib.rs");

    // Each file defines one function and calls previous one to create a chain of calls
    for i in 0..n {
        let path = src.join(format!("f{}.rs", i));
        let mut file = fs::File::create(&path).expect("create file");
        if i == 0 {
            writeln!(file, "pub fn f0() {{}} ").unwrap();
        } else {
            writeln!(file, "use crate::f{}::f{};", i - 1, i - 1).unwrap();
            writeln!(file, "pub fn f{}() {{ f{}(); }}", i, i - 1).unwrap();
        }
    }

    println!("Generated synthetic project at {} with {} files", root.display(), n);
}
