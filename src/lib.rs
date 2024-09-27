use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{PathBuf, Path};

#[derive(Default, Debug)]
pub struct GrepConfig {
    pub quiet: bool,
    pub recursive: bool,
    pub regex: bool,
}

#[derive(Default, Debug)]
pub struct ReplaceConfig {
    pub quiet: bool,
    pub recursive: bool,
    pub regex: bool,
    pub nth: usize,
    pub replace_all: bool,
    pub fill_byte: u8,
    pub allow_length_change: bool,
}

#[derive(Default, Debug)]
pub struct InsertConfig {
    pub quiet: bool,
}

#[derive(Debug)]
pub struct OpenedFile<'a, T: AsRef<Path>> {
    pub file: File,
    pub path: &'a T,
}

#[derive(Debug)]
pub struct Match<'a, T: AsRef<Path>> {
    pub opened_file: &'a mut OpenedFile<'a, T>,
    pub pattern: &'a [u8],
    pub offset: u64,
}

impl<'a, T: AsRef<Path>> Iterator for Match<'a, T> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let pattern_len = self.pattern.len();
        // This is a slow O(n^2) way to do it. Obviously we can be smarter about it,
        // using a proper string-searching algorithm
        let mut buf = vec![0u8; pattern_len];
        self.opened_file.file.seek(SeekFrom::Start(self.offset)).unwrap();
        while let Ok(()) = self.opened_file.file.read_exact(&mut buf) {
            self.offset += 1;
            if buf == self.pattern {
                return Some(self.offset-1);
            }
            self.opened_file.file.seek(SeekFrom::Start(self.offset)).unwrap();
        }
        None
    }
}

fn find_matches<'a, T: AsRef<Path>>(
    opened_file: &'a mut OpenedFile<'a, T>,
    pattern: &'a [u8]
) -> Match<'a, T> {
    Match::<'a, T>{ opened_file, pattern, offset: 0 }
}

fn open_file<T: AsRef<Path>>(filename: &T) -> Result<OpenedFile<T>, io::Error> {
    match File::open(filename) {
        Ok(f) => Ok(OpenedFile {file: f, path: filename}),
        Err(e) => {
            eprintln!("Can't open {} bacause of error: {}", filename.as_ref().display(), e);
            Err(e)
        }
    }
}

fn open_files<T: AsRef<Path>>(filenames: &[T]) -> Result<Vec<OpenedFile<T>>, io::Error> {
    let mut ret = Vec::new();
    for filename in filenames {
        let file = open_file(filename)?;
        ret.push(file);
    }
    Ok(ret)
}

pub fn grep_command<T: AsRef<Path>>(
    pattern: &[u8],
    filenames: &[T],
    grep_config: &GrepConfig,
) -> Result<(), io::Error> {
    let results = grep(pattern, filenames)?;
    if !grep_config.quiet {
        println!("Nothing found");
    }
    for (n, (filename, offsets)) in results.iter().enumerate() {
        println!("{}:", filename.display());
        for (n, offset) in offsets.iter().enumerate() {
            print!("{}", offset);
            if n != offsets.len() - 1 {
                print!(", ");
            }
        }
        if n != results.len() - 1 {
            println!("\n");
        }
    }
    
    Ok(())
}

pub fn grep<T: AsRef<Path>>(
    pattern: &[u8],
    filenames: &[T],
) -> Result<Vec<(PathBuf, Vec<u64>)>, io::Error> {
    let mut ret = Vec::new();
    let mut files = open_files(filenames)?;
    for file in files.iter_mut() {
        ret.push((PathBuf::from(file.path.as_ref()), Vec::new()));
        let found_matches: Vec<_> = find_matches(file, pattern).collect();
        for &offset in found_matches.iter() {
            ret.last_mut().unwrap().1.push(offset);
        }
    }
    
    Ok(ret)
}

pub fn replace_command(
    replace: &[u8],
    replace_with: &[u8],
    filename: &Path,
    replace_config: &ReplaceConfig,
) {
}

pub fn replace(
    replace: &[u8],
    replace_with: &[u8],
    filename: &Path,
    replace_config: &ReplaceConfig,
) -> Result<(), io::Error> {
    let file = open_file(&filename)?;
    Ok(())
}

pub fn insert_command(
    to_insert: &[u8],
    offset: usize,
    filename: &Path,
    insert_config: &InsertConfig
) {
    
}

pub fn insert(to_insert: &[u8], offset: usize, filename: &Path, insert_config: &InsertConfig) {}
