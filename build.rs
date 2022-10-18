use std::process::exit;

use build_target::Os;

fn main() {
  let target = build_target::target().expect("Unable to get compile target info");
  if target.os != Os::MacOs {
    eprintln!("Only macOS is supported target os, actual: {:#?}", target);
    exit(1);
  }
}
