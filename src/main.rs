use std::process::{self, Stdio};

use anyhow::{ensure, Result, Context};
use clap::Parser;
use clap_verbosity_flag::{LogLevel, Verbosity};
use lazy_static::lazy_static;
use log::{debug, error, LevelFilter};
use log4rs::{
  append::console::ConsoleAppender,
  config::{Appender, Root},
  encode::pattern::PatternEncoder,
};
use rdev::{listen, Event, EventType, Key, Keyboard, KeyboardState};

#[derive(Parser, Debug)]
struct Cli {
  #[clap(flatten)]
  verbose: Verbosity<DefaultLevel>,
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
  let args = Cli::parse();
  init_logger(args.verbose.log_level_filter());
  debug!("{args:?}");
  if let Err(err) = listen(on_keyboard) {
    error!("Error during listening keyboard event {err:#?}")
  };
  Ok(())
}

fn new_ctrl_alt_cmd() -> Keyboard {
  let mut key = Keyboard::new().unwrap();
  key.add(&EventType::KeyPress(Key::MetaLeft));
  key.add(&EventType::KeyPress(Key::Alt));
  key.add(&EventType::KeyPress(Key::ControlLeft));
  key
}

lazy_static! {
  static ref CTRL_OPT_CMD_UP: String = {
    new_ctrl_alt_cmd()
      .add(&EventType::KeyPress(Key::UpArrow))
      .expect("Failed to create CTRL_OPT_CMD_UP")
  };
  static ref CTRL_OPT_CMD_DOWN: String = {
    new_ctrl_alt_cmd()
      .add(&EventType::KeyPress(Key::DownArrow))
      .expect("Failed to create CTRL_OPT_CMD_DOWN")
  };
  static ref CTRL_OPT_CMD_LEFT: String = {
    new_ctrl_alt_cmd()
      .add(&EventType::KeyPress(Key::LeftArrow))
      .expect("Failed to create CTRL_OPT_CMD_LEFT")
  };
  static ref CTRL_OPT_CMD_RIGHT: String = {
    new_ctrl_alt_cmd()
      .add(&EventType::KeyPress(Key::RightArrow))
      .expect("Failed to create CTRL_OPT_CMD_RIGHT")
  };
  static ref CTRL_OPT_CMD_P: String = {
    new_ctrl_alt_cmd()
      .add(&EventType::KeyPress(Key::KeyP))
      .expect("Failed to create CTRL_OPT_CMD_P")
  };
}

fn on_keyboard(event: Event) {
  let key = if let Event {
    name,
    time: _,
    event_type: EventType::KeyPress(key),
  } = event
  {
    dbg!(&key);
    name
  } else {
    return;
  };

  let key = if let Some(key) = key {
    key
  } else {
    return;
  };

  dbg!(&key);

  let result = if key == CTRL_OPT_CMD_UP.as_str() {
    volume_up().context("Failed to let volume up")
  } else if key == CTRL_OPT_CMD_DOWN.as_str() {
    volume_down().context("Failed to let volume down")
  } else if key == CTRL_OPT_CMD_LEFT.as_str() {
    previous_track().context("Failed to switch to previous track")
  } else if key == CTRL_OPT_CMD_RIGHT.as_str() {
    next_track().context("Failed to switch to next track")
  } else if key == CTRL_OPT_CMD_P.as_str() {
    playpause().context("Failed to playpause")
  } else {
    return;
  };

  if let Err(err) = result {
    error!("{err}");
  };
}

fn osascript_js(code: &str) -> Result<()> {
  let output = process::Command::new("osascript")
    .stderr(Stdio::inherit())
    .stdout(Stdio::inherit())
    .arg("-l")
    .arg("JavaScript")
    .arg("-e")
    .arg(&code)
    .output()?;
  ensure!(output.status.success(), "Failed to execute osascript...");
  Ok(())
}

fn playpause() -> Result<()> {
  osascript_js("Application('Music').playpause()")
}

fn volume_down() -> Result<()> {
  osascript_js(
    "
    var app = Application('Music');
    var volume = app.soundVolume();
    app.soundVolume = volume - 10 > 0 ? volume - 10 : 0;
    ",
  )
}

fn volume_up() -> Result<()> {
  osascript_js(
    "
    var app = Application('Music');
    var volume = app.soundVolume();
    app.soundVolume = volume + 10 < 100 ? volume + 10 : 100;
    ",
  )
}

fn next_track() -> Result<()> {
  osascript_js("Application('Music').nextTrack();")
}

fn previous_track() -> Result<()> {
  osascript_js("Application('Music').previousTrack();")
}

#[cfg(debug_assertions)]
type DefaultLevel = DebugLevel;

#[cfg(not(debug_assertions))]
type DefaultLevel = clap_verbosity_flag::InfoLevel;

#[derive(Copy, Clone, Debug, Default)]
pub struct DebugLevel;

impl LogLevel for DebugLevel {
  fn default() -> Option<log::Level> {
    Some(log::Level::Debug)
  }
}

fn init_logger(verbosity: LevelFilter) {
  const PATTERN: &str = "{d(%m-%d %H:%M)} {h({l:.1})} - {h({m})}{n}";
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(PATTERN)))
    .build();
  let config = log4rs::Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .build(Root::builder().appender("stdout").build(verbosity))
    .unwrap();
  log4rs::init_config(config).unwrap();
}
