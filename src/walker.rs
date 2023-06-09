use std::env;
use std::path::PathBuf;
use std::path::Path;
use std::io::Write;

use ignore::WalkBuilder;
use crate::matcher::{Matcher,MatcherKind};

pub struct FileWalker {
  pub path: PathBuf,
}

impl FileWalker {
pub fn new(path: &PathBuf) -> Self {
    Self { path: path.to_path_buf() }
  }

  pub fn walk_files (&mut self, rules: &Vec<String>) -> Vec<DirEntry> {
    let (tx, rx) = flume::unbounded();  

    let walker = WalkBuilder::new(&self.path)
        .git_ignore(false)
        .ignore(false)
        .parents(false)
        .threads(6)
        .build_parallel();

    // Include all files that start with ! exclimation mark
    // let include_matches = rules.iter().filter(|x| x.starts_with("!")).map(|x| format!("{}", x)).collect::<Vec<_>>();
    
    // Exclude all files that don't start with ! exclimation mark
    // let exclude_matches = rules.iter().filter(|x| !x.starts_with("!")).map(|x| format!("{}", x)).collect::<Vec<_>>();
     
    
    // println!("Includes: {:?}", include_matches);
    // println!("Excludes: {:?}", exclude_matches);

    walker.run(|| {
      let tx = tx.clone();
      let matcher = 
        Matcher::new(
          MatcherKind::Exclude(
            Matcher::create_matcher(&env::current_dir().unwrap(), rules.to_vec()).unwrap()
        ));

      
      Box::new(move |result| {
        let dent = match result {
          Ok(dent) => { dent },
          Err(err) => {
            eprintln!("{}", err);
            return ignore::WalkState::Continue;
          }
        };
        
        if matcher.should_include(&dent.path()) {
          tx.send(DirEntry::X(dent)).unwrap();
        }

        ignore::WalkState::Continue
      })
    });

    drop(tx);
    rx.drain().collect::<Vec<_>>()
  }
}

pub enum DirEntry {
  X(ignore::DirEntry),
}

impl DirEntry {
 pub fn path(&self) -> &Path {
    match *self {
      DirEntry::X(ref y) => y.path(),
    }
  }
}

#[cfg(unix)]
fn write_path<W: Write>(mut wtr: W, path: &Path) {
  use std::os::unix::ffi::OsStrExt;
  wtr.write(path.as_os_str().as_bytes()).unwrap();
  wtr.write(b"\n").unwrap();
}

#[cfg(not(unix))]
fn write_path<W: Write>(mut wtr: W, path: &Path) {
  wtr.write(path.to_string_lossy().as_bytes()).unwrap();
  wtr.write(b"\n").unwrap();
}
