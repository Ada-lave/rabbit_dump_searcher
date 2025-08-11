use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use grep_regex::RegexMatcher;
use std::io::{self, Error};
use grep_searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use grep_searcher::{Searcher};
use memmap2::Mmap;

use crate::indices::binary::BinaryRef;
use crate::indices::content::IndexContentSink;


pub struct ProcHeapRef {
    ref_name: String,
    pub data: String
}

pub struct ProcHeapIndex {
    heap_refs: HashMap<String, ProcHeapRef>,
}

impl ProcHeapIndex {
     pub fn new() -> Self {
        Self {heap_refs: HashMap::new()}
    }
    pub fn index_proc_heap(&mut self, matches: &Vec<(Option<String>, u64)>, file_data: &[u8], binary_refs: &HashMap<String, BinaryRef>) -> Result<bool, Error> {
        let matcher = RegexMatcher::new(r"^[A-Za-z0-9]{12}:t6:A7:content*").unwrap();
        let mut searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(false)
            .build();
        for window in matches.windows(2) {
            let mut content_index = IndexContentSink::new();
            let (_, offset1) = &window[0];
            let (_, offset2) = &window[1];
            let data_slice = &file_data[*offset1 as usize..*offset2 as usize];
            searcher.search_slice(&matcher, data_slice, &mut content_index).unwrap();
            let messages = content_index.get_messages(file_data, binary_refs).unwrap();
            if messages.len() > 0 {
                let file = File::create(format!("out.json")).unwrap();
                serde_json::to_writer_pretty(file, &messages).unwrap();
                break;
            }  
        }
        Ok(true)
    }
}
