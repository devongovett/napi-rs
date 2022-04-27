use log::{max_level, Log, Metadata, Record};

pub struct SimpleLogger;

impl Log for SimpleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= max_level()
  }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      println!("{}", record.args());
    }
  }

  fn flush(&self) {}
}

pub use log::*;
