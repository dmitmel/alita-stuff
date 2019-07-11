use failure::{Fallible, ResultExt};
use log::info;

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
  pub fn init(config: crate::config::DatabaseConfig) -> Fallible<Self> {
    let path: &Path = &config.path;
    let file_exists = path.exists();

    info!("opening file '{}'", path.display());
    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(path)
      .context("failed to open file")?;

    let mut db = Self { file, records: vec![] };

    if file_exists {
      info!("reading data");
      db.read()?;
    } else {
      info!("writing default data to the file");
      db.write()?;
    }

    Ok(db)
  }

  pub fn records(&self) -> &[Record] {
    &self.records
  }

  pub fn compress_records<F>(&self, mut callback: F)
  where
    F: FnMut(&Record),
  {
    if self.records.is_empty() {
      return;
    }

    let first_record = &self.records[0];
    callback(first_record);

    if self.records.len() == 1 {
      return;
    }

    let mut prev_record = first_record;
    let mut prev_record_had_changes = true;

    for record in &self.records[1..] {
      macro_rules! record_has_changes {
        ($($field:ident),+ $(,)?) => {
          $(record.$field != prev_record.$field)||+
        };
      }

      if record_has_changes![rank, upvotes, downvotes, reranks, top5_reranks] {
        if !prev_record_had_changes {
          callback(prev_record);
        }
        prev_record_had_changes = true;
        callback(record);
      } else {
        prev_record_had_changes = false;
      }

      prev_record = record;
    }

    if !prev_record_had_changes {
      callback(prev_record);
    }
  }

  pub fn push(&mut self, record: Record) -> Fallible<()> {
    self.file.seek(SeekFrom::End(0))?;

    let mut writer = BufWriter::new(&self.file);

    serde_json::to_writer(&mut writer, &record)?;
    writer.write_all(b"\n")?;
    self.records.push(record);

    info!("pushed record #{}", self.records.len());
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
        format!("failed to deserialize line {}: {:?}", line_number, line)
      })?;
      self.records.push(record);
      line.clear();
      line_number += 1;
    }

    info!("read {} records", self.records.len());
    Ok(())
  }

  pub fn write(&mut self) -> Fallible<()> {
    self.file.seek(SeekFrom::Start(0))?;

    let mut writer = BufWriter::new(&self.file);
    for record in &self.records {
      serde_json::to_writer(&mut writer, &record)
        .with_context(|_| format!("failed to serialize record {:?}", record))?;
      writer.write_all(b"\n")?;
    }

    info!("written {} records", self.records.len());
    Ok(())
  }
}
