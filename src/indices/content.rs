use std::collections::HashMap;
use grep_regex::{Error, RegexMatcher};
use grep_searcher::{BinaryDetection, SearcherBuilder};

use crate::indices::binary::BinaryRef;
use crate::indices::global::IndexSink;


#[derive(Debug)]
enum NodeValue {
    Link { left: String, right: Option<String> },
    Data { address: String, offset: usize, size: usize },
}
pub struct ContentIndex<'a> {
    pub index_sink: IndexSink,
    pub mmap: &'a [u8]

}


impl <'a>ContentIndex<'a> {
    pub fn new(mmap: &'a [u8]) -> Self {
        Self {
            mmap: mmap,
            index_sink: IndexSink::new(),
        }
    }

    pub fn build(&mut self) -> Result<(), Error> {
        let matcher = RegexMatcher::new(r"t6:A7:content*")?;
        let mut searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(false)
            .build();
        match searcher.search_slice(matcher, &self.mmap, &mut self.index_sink) {
            Ok(_) => {},
            Err(error) => panic!("Failed to build ContentIndex: {error:?}")
        };
        Ok(())
    }

    pub fn get_messages(&self, binary_refs: &'a HashMap<String, BinaryRef>) -> Result<Vec<String>, Error> {
        let mut messages: Vec<String> = Vec::new();
        for window in self.index_sink.matches.windows(2) {
            let (_, offset1) = &window[0];
            let (_, offset2) = &window[1];
            let mut message_parts: Vec<&'a str> = Vec::new();
            let data_slice = &self.mmap[*offset1 as usize..*offset2 as usize];
            let proc_heap_data = std::str::from_utf8(data_slice).unwrap();
            let content_nodes = self.build_content_map(proc_heap_data);
            for node in &content_nodes {
                match node {
                    NodeValue::Data { address, offset, size } => {
                        let part = self.find_message_part(binary_refs, address.to_string());
                        match part {
                            Some(part) => {
                                message_parts.push(part);
                            }
                            None => {}
                        }
                    }
                    NodeValue::Link { left, right } => {}
                }
            }
            let data = message_parts.join("");
            if data.len() > 2 {
                messages.push(data);
            }
        }
        if let Some(last_match) = self.index_sink.matches.last() {
            let (_, offset1) = last_match;
            let mut message_parts: Vec<&'a str> = Vec::new();
            let data_slice = &self.mmap[*offset1 as usize..*&self.mmap.len() as usize];
            let proc_heap_data = std::str::from_utf8(data_slice).unwrap();
            let content_nodes = self.build_content_map(proc_heap_data);
            for node in &content_nodes {
                match node {
                    NodeValue::Data { address, offset, size } => {
                        let part = self.find_message_part(binary_refs, address.to_string());
                        match part {
                            Some(part) => {
                                message_parts.push(part);
                            }
                            None => {}
                        }
                    }
                    NodeValue::Link { left, right } => {}
                }
            }
            messages.push(message_parts.join(""));
        }
        Ok(messages)
    }

    fn find_message_part(&self, binary_refs: &'a HashMap<String, BinaryRef>, key: String) -> Option<&'a str> {
        let bin_ref = binary_refs.get(&key);
        match bin_ref {
            Some(bin_ref) => {
                return Some(bin_ref.binary_data.data);
            }
            None => {return None}
        }
    }

    fn build_content_map(&self, input: &str) -> Vec<NodeValue> {
        let mut data_nodes: Vec<NodeValue> = Vec::new();
        for line in input.trim().lines() {
            if let Some((key, value)) = line.split_once(':') {
                if value.starts_with("Yc") {
                    let parts: Vec<&str> = value.split(':').collect();
                    if parts.len() >= 3 {
                        data_nodes.push(
                            NodeValue::Data {
                                address: parts[0].trim_start_matches("Yc").to_string(),
                                offset: usize::from_str_radix(parts[1], 16).unwrap_or(0),
                                size: usize::from_str_radix(parts[2], 16).unwrap_or(0),
                            },
                        );
                    }
                }
            }
            
        }
        return data_nodes;
    }
}

#[test]
fn build_index() {
    use std::{fs::File}; 
    use memmap2::Mmap;

    const FILE_NAME: &str = "sample_data/content.txt";
    let file = File::open(FILE_NAME).unwrap();
    let mmap = unsafe {
        Mmap::map(&file).unwrap()
    };
    let mut content_index = ContentIndex::new(&mmap);
    content_index.build().unwrap();
    content_index.get_messages(&HashMap::new()).unwrap();
}