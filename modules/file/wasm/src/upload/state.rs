use std::io::Read;

pub(crate) struct UploadingState {
    pub file: std::fs::File,
    pub len: usize,
    pub uploaded: usize,
}

impl UploadingState {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;

        let len = file.metadata()?.len();

        Ok(Self {
            file,
            len: len as usize,
            uploaded: 0
        })
    }

    pub fn next_chunk(&mut self, size: usize) -> Option<Vec<u8>> {
        if self.uploaded >= self.len {
            return None
        }

        let chunk_size = std::cmp::min(size, self.len - self.uploaded);
        let mut chunk = vec![0u8; chunk_size];

        let Ok(bytes_read) = self.file.read(&mut chunk) else {
            return None
        };

        self.uploaded += bytes_read;

        Some(chunk)
    }
}

pub(crate) enum UploadState {
    NotStarted,
    Uploading(UploadingState),
    Finished,
}