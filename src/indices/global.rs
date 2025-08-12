
use std::{io};
use grep_searcher::{Sink, SinkMatch};
use grep_searcher::{Searcher};


pub struct IndexSink {
    pub matches: Vec<(Option<String>, u64)>,
}


impl IndexSink {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
        }
    }
}

impl Sink for IndexSink {
    type Error = io::Error;
    fn matched(&mut self, _searcher: &Searcher, match_: &SinkMatch) -> Result<bool, Self::Error> {
        let byte_offset = match_.absolute_byte_offset();
        let match_bytes = match_.bytes();
        let tag_end = match_bytes
                .iter()
                .position(|&x| x == b':')
                .unwrap_or(match_bytes.len() - 1);
        let tag_id_string = if match_bytes.len() > tag_end + 1 {
            let tag_id_cow = String::from_utf8_lossy(&match_bytes[tag_end + 1..]);
            Some(tag_id_cow.trim().to_string())
        } else {
            None
        };

        self.matches.push((tag_id_string, byte_offset));
        Ok(true)
    }    
}