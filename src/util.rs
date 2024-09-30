use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{PathBuf, Path};

/// Custom struct to bundle an opened file and its path together
#[derive(Debug)]
pub struct OpenedFile<'a> {
    pub file: File,
    pub path: &'a Path,
}

/// Iterator returned by the `find_matches()` function. It helps us to
/// get all the offsets of the matches of a pattern in an opened file.
#[derive(Debug)]
pub struct Match<'a> {
    pub opened_file: &'a mut OpenedFile<'a>,
    pub pattern: &'a [u8],
    pub offset: u64,
}

/// Iterator returned by the `open_files()` function. Avoid using
/// `.collect()`, otherwise we will hit the opened file descriptors
/// limit.
#[derive(Debug)]
pub struct OpenFiles<'a, T: AsRef<Path> + 'a> {
    pub files: &'a [T],
    pub nth: usize,
}

impl<'a, T: AsRef<Path> + 'a> Iterator for OpenFiles<'a, T> {
    type Item = OpenedFile<'a>;
    fn next(&mut self) -> Option<OpenedFile<'a>> {
        match self.files.get(self.nth) {
            Some(filename) => {
                self.nth += 1;
                open_file(filename.as_ref()).ok()
            },
            None => None,
        }
    }
}

impl<'a> Iterator for Match<'a> {
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

pub fn find_matches<'a>(
    opened_file: &'a mut OpenedFile<'a>,
    pattern: &'a [u8]
) -> Match<'a> {
    Match::<'a>{ opened_file, pattern, offset: 0 }
}

pub fn open_file<'a>(filename: &'a Path) -> Result<OpenedFile<'a>, io::Error> {
    match File::open(&filename) {
        Ok(f) => Ok(OpenedFile {file: f, path: filename}),
        Err(e) => {
            eprintln!("Can't open {} bacause of error: {}", filename.display(), e);
            Err(e)
        }
    }
}

pub fn open_files<'a, T: AsRef<Path>>(filenames: &'a [T]) -> OpenFiles<'a, T> {
    OpenFiles { files: filenames, nth: 0 }
}

/// Open a directory recursively, getting all the files in the
/// directory and its subdirectories. Doesn't work with symlinks. We
/// make an assumption that the dir argument is a directory.
fn open_recursively(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut ret = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            ret.push(entry.path());
        } else if metadata.is_dir() {
            ret.append(&mut open_recursively(&entry.path())?);
        }
    }
    
    Ok(ret)
}

/// Same as `open_recursively()`, except we do it for every path in a
/// slice. A path doesn't need to be a directory, it can be a file -
/// then it's just added to the returned Vec.
pub fn open_all_directories<T: AsRef<Path>>(paths: &[T]) -> Result<Vec<PathBuf>, io::Error> {
    let mut ret = Vec::new();
    for path in paths {
        if path.as_ref().is_dir() {
            ret.append(&mut open_recursively(path.as_ref())?);
        } else if path.as_ref().is_file() {
            ret.push(path.as_ref().to_path_buf());
        }
    }
    
    Ok(ret)
}
