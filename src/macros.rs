use crate::SETTINGS;

pub fn check_settings() -> bool {
  let settings = SETTINGS.read().unwrap();
  if let Ok(enabled) = settings.get_bool("performance.enabled") { return enabled; }
  else { return false; }
}

mod macros {
#[macro_export]
  macro_rules! perf {
    ($($arg:tt)*) => (
      if crate::macros::check_settings() { 
        use std::io::Write as _;
        use std::fs::{OpenOptions};
        let settings = crate::SETTINGS.read().unwrap();

        if let Ok(path) = settings.get_str("performance.path") {
          let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .unwrap();

          if let Err(e) = writeln!(file, $($arg)*) {
            eprintln!("Couldn't write to file: {}", e);
          }
        }}
        )
  }

  #[macro_export]
  macro_rules! perfend {
    ($($arg:tt)*) => (
      if crate::macros::check_settings() { 
        use std::io::Write as _;
        use std::fs::{OpenOptions};
        let settings = crate::SETTINGS.read().unwrap();

        if let Ok(path) = settings.get_str("performance.path") {
          let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .unwrap();

          if let Err(e) = writeln!(file, $($arg)*) {
            eprintln!("Couldn't write to file: {}", e);
          }
        }}
        else {
          println!($($arg)*);
        })
  }
}
