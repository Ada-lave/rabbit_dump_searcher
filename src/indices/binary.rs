use grep_regex::RegexMatcher;
use grep_searcher::{BinaryDetection, SearcherBuilder};
use memmap2::Mmap;

use std::collections::HashMap;
use std::io::Error;

use crate::indices::global::IndexSink;
pub struct BinaryData<'a> {
    pub size: usize,
    pub data: &'a str
}

pub struct RabbitFraming {
    pub ref_name: String,
    pub message_parts: String 
}

pub struct BinaryRef<'a> {
    pub ref_name: String,
    pub binary_data: BinaryData<'a>
}

pub struct BinaryIndex<'a> {
    pub total_size: usize,
    pub index_sink: IndexSink,
    pub binary_refs: HashMap<String, BinaryRef<'a>>,
    pub mmap: &'a [u8]

}

impl <'a> BinaryIndex <'a> {
    pub fn new(mmap: &'a [u8]) -> Self {
        Self {mmap: mmap, total_size:0, index_sink: IndexSink::new(), binary_refs: HashMap::new() }
    }

    pub fn build(&mut self) {
        let matcher = RegexMatcher::new(r"^=binary.*").unwrap();
        let mut searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(false)
            .build();
        searcher.search_slice(matcher, &self.mmap, &mut self.index_sink).unwrap();
    }

    pub fn index(&mut self) -> Result<bool, Error> {
        for window in self.index_sink.matches.windows(2) {
            let (_, offset1) = &window[0];
            let (_, offset2) = &window[1];
            let data_slice = &self.mmap[*offset1 as usize..*offset2 as usize];
            let mut lines = std::str::from_utf8(data_slice).unwrap().lines();
            
            if let Some((_, part2)) = lines.next().unwrap().split_once(':') {
                if let Some((hex_size, data)) = lines.next().unwrap().split_once(':') {
                    let size = usize::from_str_radix(hex_size, 16).unwrap();  
                    self.binary_refs.insert(part2.to_string(), 
                        BinaryRef {
                            ref_name: part2.to_string(),
                            binary_data: BinaryData {
                                size,
                                data: data,
                            }
                        }
                    );
                    self.total_size += size;
                }
            }
        }  
        Ok(true)
    }

    pub fn get_ref_by_name(&self, ref_name: &str) -> Option<&BinaryRef<'_>> {
        return self.binary_refs.get(ref_name);
    }
}