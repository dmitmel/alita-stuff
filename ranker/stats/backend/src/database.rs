use failure::{Fallible, ResultExt};

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

use crate::record::Record;

#[derive(Debug)]
pub struct Database {
  file: File,
  records: Vec<Record>,
}

impl Database {
  pub fn init(path: &Path) -> Fallible<Self> {
    let file_exists = path.exists();

    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(path)
      .with_context(|_| format!("couldn't open file '{}'", path.display()))?;

    let mut state = Self {
      file,
      records: vec![],
    };

    if file_exists {
      state.read()?;
    } else {
      state.write()?;
    }

    Ok(state)
  }

  pub fn push(&mut self, record: Record) -> Fallible<()> {
    self.file.seek(SeekFrom::End(0))?;

    let mut writer = BufWriter::new(&self.file);

    serde_json::to_writer(&mut writer, &record)?;
    writer.write_all(b"\n")?;
    self.records.push(record);

    Ok(())
  }

  pub fn read(&mut self) -> Fallible<()> {
    self.file.seek(SeekFrom::Start(0))?;

    self.records = vec![];

    let mut reader = BufReader::new(&self.file);
    let mut line_number = 1;
    let mut line = String::with_capacity(128);
    while reader.read_line(&mut line)? > 0 {
      let record = serde_json::from_str(&line).with_context(|_| {
        format!("couldn't deserialize line {}: {:?}", line_number, line)
      })?;
      self.records.push(record);
      line.clear();
      line_number += 1;
    }

    Ok(())
  }

  pub fn write(&mut self) -> Fallible<()> {
    self.file.seek(SeekFrom::Start(0))?;

    let mut writer = BufWriter::new(&self.file);
    for record in &self.records {
      serde_json::to_writer(&mut writer, &record)
        .with_context(|_| format!("couldn't serialize record {:?}", record))?;
      writer.write_all(b"\n")?;
    }

    Ok(())
  }
}
