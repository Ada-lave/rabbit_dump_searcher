use std::collections::HashMap;
use std::io;
use std::io::{Error};
use grep_searcher::{Sink, SinkMatch};
use grep_searcher::{Searcher};

use crate::indices::binary::BinaryRef;

pub struct IndexContentSink {
    pub content_matches: Vec<(Option<String>, u64)>,
}


impl IndexContentSink {
    pub fn new() -> Self {
        Self {
            content_matches: Vec::new()
        }
    }

    pub fn get_messages(&self, file_data: &[u8], binary_refs: &HashMap<String, BinaryRef>) -> Result<Vec<String>, Error> {
        let mut messages: Vec<String> = Vec::new();
        println!("content_matches len: {}", self.content_matches.len());
        for window in self.content_matches.windows(2) {
            let (_, offset1) = &window[0];
            let (_, offset2) = &window[1];
            let mut message_parts: Vec<String> = Vec::new();
            let data_slice = &file_data[*offset1 as usize..*offset2 as usize];
            let mut proc_heap_data = std::str::from_utf8(data_slice).unwrap().lines();
            let mut content_data_ref = proc_heap_data.next();
            
            loop {
                match content_data_ref {
                    Some(data) => {
                        if !data.contains(':') && data.contains("content") {
                            content_data_ref = proc_heap_data.next();
                            continue;
                        }
                        println!("{}", data);
                        let (_, list) = data.split_once(':').unwrap();
                        let (young_heap_reff, next_data_reff) = list.split_once(':').unwrap();
                        message_parts.push(self.find_message_part(binary_refs, young_heap_reff.to_string()));
                        if next_data_reff != "N" {
                            content_data_ref = proc_heap_data.next();
                        }
                    }
                    None => break
                }
            }
            messages.push(message_parts.join(""));
        }
        Ok(messages)
    }

    fn find_message_part(&self, binary_refs: &HashMap<String, BinaryRef>, key: String) -> String {
        let bin_ref = binary_refs.get(&key);
        return bin_ref.unwrap().binary_data.data.to_string();
    }
}

impl Sink for IndexContentSink {
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
        self.content_matches.push((tag_id_string, byte_offset));
        
        Ok(true)
    }    
}