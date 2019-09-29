use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct Record<T> {
  pub timestamp: Timestamp,
  pub data: T,
}

pub struct Timestamp {
  secs: i64,
  tm: time::Tm,
}

impl Timestamp {
  pub fn new(secs: i64) -> Self {
    Self { secs, tm: time::at_utc(time::Timespec::new(secs, 0)) }
  }

  pub fn now() -> Self {
    Self::new(time::get_time().sec)
  }

  pub fn as_secs(&self) -> i64 {
    self.secs
  }

  pub fn format_to<W: Write>(&self, mut wr: W) -> io::Result<()> {
    fn write_padded_i32<W: Write>(
      mut wr: W,
      value: i32,
      len: usize,
    ) -> io::Result<()> {
      let mut abs_str_buf = itoa::Buffer::new();
      let abs_str: &str = abs_str_buf.format(value.abs());
      let mut padding_len = len.saturating_sub(abs_str.len());
      if value < 0 {
        padding_len = padding_len.saturating_sub(1);
        wr.write_all(b"-")?;
      }
      for _ in 0..padding_len {
        wr.write_all(b"0")?;
      }
      wr.write_all(abs_str.as_bytes())
    }

    let tm = self.tm;

    itoa::write(&mut wr, tm.tm_year + 1900)?;
    wr.write_all(b"-")?;
    write_padded_i32(&mut wr, tm.tm_mon + 1, 2)?;
    wr.write_all(b"-")?;
    write_padded_i32(&mut wr, tm.tm_mday, 2)?;
    wr.write_all(b" ")?;
    write_padded_i32(&mut wr, tm.tm_hour, 2)?;
    wr.write_all(b":")?;
    write_padded_i32(&mut wr, tm.tm_min, 2)?;
    wr.write_all(b":")?;
    write_padded_i32(&mut wr, tm.tm_sec, 2)?;

    Ok(())
  }
}

use std::fmt;
impl fmt::Debug for Timestamp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.secs.fmt(f)
  }
}

impl<'de> Deserialize<'de> for Timestamp {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    i64::deserialize(deserializer).map(Timestamp::new)
  }
}

impl Serialize for Timestamp {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.secs.serialize(serializer)
  }
}
