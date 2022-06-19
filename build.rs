use std::process::Command;


/// Cargo should rebuild the grammar whenever I change it
fn main() {
  println!("cargo:rerun-if-changed=src/parser/Msh.g4");
  Command::new("java").args(["-jar", "./antlr4.jar",
      "-Dlanguage=Rust", "-visitor", "src/parser/Msh.g4"]).output().expect("failure generating the grammar");
}