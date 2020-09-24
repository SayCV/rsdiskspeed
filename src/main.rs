#![allow(unused_imports)]

use clap::{App, Arg};
use std::{
  fs::File,
  fs::OpenOptions,
  io::{ErrorKind, Read, Seek, SeekFrom, Write},
  process::exit,
};

use rand::distributions::{Standard, Uniform};
use rand::{thread_rng, Rng};
use time::*;

mod error;
pub use error::Error;

pub type Result<T> = ::std::result::Result<T, error::Error>;

#[derive(Debug)]
pub struct Benchmark {
  file: String,
  write_mb: usize,
  write_block_kb: usize,
  read_block_kb: usize,
  write_results: Vec<f64>,
  read_results: Vec<f64>,
}

impl Benchmark {
  pub fn new(
    file: String,
    size: usize,
    write_block_kb: usize,
    read_block_kb: usize,
  ) -> Result<Benchmark> {
    Ok(Benchmark {
      file,
      write_mb: size,
      write_block_kb,
      read_block_kb,
      write_results: Vec::new(),
      read_results: Vec::new(),
    })
  }

  pub fn write_test(
    &mut self,
    block_size: usize,
    blocks_count: usize,
    show_progress: bool,
  ) -> Result<()> {
    let mut f = File::create(&self.file)?;
    self.write_results.clear();
    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("Write: {wide_bar} {pos}/{len} {msg}")
        .progress_chars("#>-"));

    for i in 0..blocks_count {
      if show_progress {
        pb.set_position(((i + 1) * 100 / blocks_count) as u64);
      }
      let rng = thread_rng();
      let buff: Vec<u8> = rng.sample_iter::<u8, _>(Standard).take(block_size).collect();
      let start = OffsetDateTime::now_utc();
      f.write_all(&buff[..])?;
      f.sync_all()?;
      let t = OffsetDateTime::now_utc() - start;
      self.write_results.push(t.as_seconds_f64());
    }

    pb.finish_with_message("done");
    drop(f);
    Ok(())
  }

  pub fn read_test(
    &mut self,
    block_size: usize,
    blocks_count: usize,
    show_progress: bool,
  ) -> Result<()> {
    let mut f = File::open(&self.file).unwrap();
    let rng = thread_rng();
    let die_range = Uniform::new_inclusive(0, blocks_count * block_size);
    let offsets: Vec<_> = rng.sample_iter(die_range).take(blocks_count).collect();
    self.read_results.clear();
    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("Read: {wide_bar} {pos}/{len} {msg}")
        .progress_chars("#>-"));

    for (i, &offset) in offsets.iter().enumerate() {
      if show_progress && i % (self.write_block_kb / self.read_block_kb) as usize == 0 {
        pb.set_position(((i + 1) * 100 / blocks_count) as u64);
      }
      // let mut rng = thread_rng();
      // let buff: Vec<u8> = rng.sample_iter(Standard).take(block_size).collect();
      let mut buff: Vec<u8> = Vec::with_capacity(block_size);
      for i in 0..block_size {
        buff.push(i as u8);
      }
      let start = OffsetDateTime::now_utc();
      f.seek(SeekFrom::Start(offset as u64))?;
      let _bytes_read = f.read(&mut buff[..])?;
      //f.sync_all()?;
      let t = OffsetDateTime::now_utc() - start;
      if _bytes_read == 0 {
        //break;
      }
      self.read_results.push(t.as_seconds_f64());
    };

    pb.finish_with_message("done");
    drop(f);
    Ok(())
  }

  pub fn print_result(self) {
    let wr_sec: f64 = self.write_results.iter().sum();
    let rd_sec: f64 = self.read_results.iter().sum();
    let mut result = format!("
\n\nWritten {} MB in {:.4} s\nWrite speed is  {:.2} MB/s
\n  max: {:.2}, min: {:.2}\n",
    self.write_mb, wr_sec, self.write_mb as f64 / wr_sec,
    self.write_block_kb as f64 / (1024.0 * self.write_results.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()),
    self.write_block_kb as f64 / (1024.0 * self.write_results.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()));
    result.push_str(format!("
\nRead {} x {} KB blocks in {:.4} s\nRead speed is  {:.2} MB/s
\n  max: {:.2}, min: {:.2}\n",
    self.read_results.len(), self.read_block_kb,
    rd_sec, self.write_mb as f64 / rd_sec,
    self.read_block_kb as f64 / (1024.0 * self.read_results.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()),
    self.read_block_kb as f64 / (1024.0 * self.read_results.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap())).as_str());

    println!("{}", result);
    //Ok(())
  }

}

fn drop_caches() {
  OpenOptions::new()
      .write(true)
      .create(false)
      .open("/proc/sys/vm/drop_caches")
      .and_then(|mut f| f.write(b"3"))
      .unwrap();
}

fn main() {

  pretty_env_logger::try_init_timed().unwrap();

  let matches = App::new("rsdiskspeed")
    .about("Test your hard drive read-write speed")
    .arg(
      Arg::with_name("file")
        .help("The file to read/write to")
        .default_value("/tmp/rsdspdtest")
        .short("f"),
    )
    .arg(
      Arg::with_name("size")
        .help("Total MB to write")
        .default_value("128")
        .short("s"),
    )
    .arg(
      Arg::with_name("write-block-size")
        .help("The block size for writing in kbytes")
        .default_value("1024")
        .short("w"),
    )
    .arg(
      Arg::with_name("read-block-size")
        .help("The block size for reading in kbytes")
        .default_value("1024")
        .short("r"),
    )
    .arg(
      Arg::with_name("json")
        .help("Output to json file")
        .short("j"),
    )
    .arg(
      Arg::with_name("verbose")
        .help("Show progress")
        .default_value("true")
        .short("v"),
    )
    .get_matches();

  log::info!("Command line: {}", std::env::args().collect::<Vec<String>>().join(" "));
  
  let file: String = matches.value_of("file").unwrap().to_string();
  let size: usize = matches.value_of("size").unwrap().trim().parse().unwrap_or_else(|_| {log::error!("could not parse {:?} as size", matches.value_of("size"));exit(0x1);});
  let write_block_size: usize = matches.value_of("write-block-size").unwrap().trim().parse().unwrap();
  let read_block_size: usize = matches.value_of("read-block-size").unwrap().trim().parse().unwrap();
  let verbose: bool = matches.value_of("verbose").unwrap().parse().unwrap();

  log::info!("Try write and read file: {}, size: {} MB", file, size);

  if let Ok(mut benchmark) = Benchmark::new(file, size, write_block_size, read_block_size){
    let wr_blocks = size * 1024 / write_block_size;
    let rd_blocks = size * 1024 / read_block_size;
    benchmark.write_test( 1024 * write_block_size, wr_blocks, verbose).unwrap_or_else(|e| {log::error!("{}", e.to_string());exit(0x1);});
    if verbose { println!(""); }
    if cfg!(target_os = "linux") {  drop_caches(); }
    benchmark.read_test( 1024 * read_block_size, rd_blocks, verbose).unwrap();
    benchmark.print_result();
    log::info!("Done!");
    exit(0x0);
  };
  exit(0x1);
}
