use std::{collections::HashMap, ffi::OsString, path::PathBuf};

use clap::ValueEnum;
use crossbeam_channel::Sender;
use human_bytes::human_bytes;
use ignore::{DirEntry, ParallelVisitor, ParallelVisitorBuilder, WalkState};

#[derive(Debug, Copy, Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum DataType {
  FileSize,
  NumFiles,
}

pub struct FileSizeVisitorBuilder {
  sender: Sender<DirEntry>,
}

impl FileSizeVisitorBuilder {
  pub fn new(sender: Sender<DirEntry>) -> FileSizeVisitorBuilder {
    FileSizeVisitorBuilder { sender }
  }
}

impl<'s> ParallelVisitorBuilder<'s> for FileSizeVisitorBuilder {
  fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
    Box::new(FileSizeVisitor::new(self.sender.clone()))
  }
}

struct FileSizeVisitor {
  sender: Sender<DirEntry>,
}

impl FileSizeVisitor {
  pub fn new(sender: Sender<DirEntry>) -> FileSizeVisitor {
    FileSizeVisitor { sender }
  }
}

impl ParallelVisitor for FileSizeVisitor {
  fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> WalkState {
    match entry {
      Ok(entry) => self.sender.send(entry).unwrap(),
      Err(err) => println!("ERROR: {}", err),
    }
    WalkState::Continue
  }
}

pub struct PathSizeRecorder {
  pub root: PathBuf,
  pub data_type: DataType,
  pub data: PathSizeRecord,
}

impl PathSizeRecorder {
  pub fn new(root: PathBuf, data_type: DataType) -> PathSizeRecorder {
    PathSizeRecorder {
      root,
      data_type,
      data: PathSizeRecord::new(data_type),
    }
  }

  pub fn merge_entry(&mut self, entry: DirEntry) {
    let size = get_size(self.data_type, &entry);
    self.data.size += size;
    let mut curr_children = &mut self.data.children;
    let components =
      entry.path().strip_prefix(&self.root).unwrap().components();
    for c in components {
      let child_record = curr_children
        .entry(c.as_os_str().into())
        .or_insert(PathSizeRecord::new(self.data_type));
      child_record.size += size;
      curr_children = &mut child_record.children;
    }
  }
}

pub struct PathSizeRecord {
  pub size: u64,
  pub data_type: DataType,
  pub children: HashMap<OsString, PathSizeRecord>,
}

impl PathSizeRecord {
  fn new(data_type: DataType) -> PathSizeRecord {
    PathSizeRecord {
      size: 0,
      data_type,
      children: HashMap::default(),
    }
  }
}

fn get_size(data_type: DataType, entry: &DirEntry) -> u64 {
  match data_type {
    DataType::FileSize => match entry.metadata() {
      Ok(md) => md.len(),
      Err(err) => {
        println!("{}: {}", entry.path().display(), err.to_string());
        return 0;
      },
    },
    DataType::NumFiles => 1,
  }
}

pub fn data_string(data_type: DataType, size: u64) -> String {
  match data_type {
    DataType::FileSize => human_bytes(size as f64),
    DataType::NumFiles => size.to_string(),
  }
}
