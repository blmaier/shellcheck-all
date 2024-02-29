use ignore::{Walk, WalkBuilder};
use std::path::Path;

trait WalkBuilderExt {
    fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(iter: I) -> Self;
}

impl WalkBuilderExt for WalkBuilder {
    fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut iter_mut = iter.into_iter();
        let mut builder = WalkBuilder::new(iter_mut.next().unwrap());

        for path in iter_mut {
            builder.add(path);
        }

        builder
    }
}

pub struct WalkShellScript {
    builder: WalkBuilder,
}

impl WalkShellScript {
    pub fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut iter_mut = iter.into_iter();
        let mut builder = WalkBuilder::new(iter_mut.next().unwrap());

        for path in iter_mut {
            builder.add(path);
        }

        WalkShellScript { builder }
    }
}

impl IntoIterator for WalkShellScript {
    type Item = Result<ignore::DirEntry, ignore::Error>;
    type IntoIter = WalkShellScriptIterator;

    fn into_iter(self) -> Self::IntoIter {
        WalkShellScriptIterator {
            walk: self.builder.build(),
        }
    }
}

pub struct WalkShellScriptIterator {
    walk: Walk,
}

impl Iterator for WalkShellScriptIterator {
    type Item = Result<ignore::DirEntry, ignore::Error>;
    fn next(&mut self) -> Option<Result<ignore::DirEntry, ignore::Error>> {
        loop {
            match self.walk.next() {
                Some(result) => match result {
                    Ok(entry) => {
                        if entry_is_shellscript(&entry) {
                            break Some(Ok(entry));
                        }
                    }
                    Err(x) => break Some(Err(x)),
                },
                None => break None,
            }
        }
    }
}

fn entry_is_file(entry: &ignore::DirEntry) -> bool {
    match entry.file_type() {
        Some(x) => x.is_file(),
        None => false,
    }
}

fn entry_is_shellscript(entry: &ignore::DirEntry) -> bool {
    if entry_is_file(entry) {
        match file_format::FileFormat::from_file(entry.path()) {
            Ok(fmt) => match fmt {
                file_format::FileFormat::ShellScript => true,
                _ => false,
            },
            Err(err) => panic!("File Format Error: {}", err),
        }
    } else {
        false
    }
}
