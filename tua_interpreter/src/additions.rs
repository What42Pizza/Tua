use crate::prelude::*;



pub trait PathBufAdditions {
    fn push_inline<P: AsRef<Path>> (self, path: P) -> Self;
}

impl PathBufAdditions for PathBuf {
    fn push_inline<P: AsRef<Path>> (mut self, path: P) -> Self {
        self.push(path);
        self
    }
}
