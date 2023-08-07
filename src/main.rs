use clap::Parser;

use dui::{
  cli::CrosstermCli,
  crawl::{DataType, FileSizeVisitorBuilder, PathSizeRecorder},
};
use ignore::WalkBuilder;
use std::{io, path::PathBuf, thread};

fn main() {
  let args = Args::parse();
  let recorder_path = args.path.clone();

  let mut stdout = io::stdout();

  let (sender, receiver) = crossbeam_channel::unbounded();
  let mut visitor_builder = FileSizeVisitorBuilder::new(sender);

  let print_thread = thread::spawn(move || {
    let mut recorder = PathSizeRecorder::new(recorder_path, args.data_type);
    loop {
      match receiver.recv() {
        Ok(entry) => recorder.merge_entry(entry),
        Err(_) => break,
      }
    }
    let cli = CrosstermCli::from_recorder(recorder);
    cli.run(&mut stdout).unwrap();
  });

  WalkBuilder::new(args.path)
    .git_ignore(false)
    .build_parallel()
    .visit(&mut visitor_builder);

  drop(visitor_builder);

  print_thread.join().unwrap();
}

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
  #[arg(value_parser, default_value = ".")]
  path: PathBuf,
  #[arg(short = 't', long = "type", value_enum, default_value = "file_size")]
  data_type: DataType,
}
