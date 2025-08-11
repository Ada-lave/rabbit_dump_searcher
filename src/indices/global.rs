use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::os::unix::fs::FileExt;
use std::{fs, io};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use grep_regex::RegexMatcher;
use grep_searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use grep_searcher::{Searcher};
use memmap2::Mmap;

use crate::indices::binary::{BinaryData, BinaryIndex, BinaryRef};
use crate::indices::proc_heap::ProcHeapIndex;


pub struct GlobalIndex<'a> {
    pub index_sink: IndexSink,
    pub proc_heap_index: ProcHeapIndex,
    pub binary_index: BinaryIndex<'a>,
    pub mmap: Mmap
}

pub struct IndexSink {
    pub proc_heap_matches: Vec<(Option<String>, u64)>,
    pub binary_matches: Vec<(Option<String>, u64)>,
}


impl IndexSink {
    fn new() -> Self {
        Self {
            proc_heap_matches: Vec::new(),
            binary_matches: Vec::new(),
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

        if match_bytes.starts_with(b"=proc_heap") {
            self.proc_heap_matches.push((tag_id_string, byte_offset));
        } else if match_bytes.starts_with(b"=binary") {
            self.binary_matches.push((tag_id_string, byte_offset));
        }
        
        Ok(true)
    }    
}

impl <'a> GlobalIndex <'a> {
    pub fn new(file: &'a File) -> Self {
        let mmap = unsafe {
            Mmap::map(file).unwrap()
        };
        Self {
            proc_heap_index: ProcHeapIndex::new(),
            binary_index: BinaryIndex::new(),
            index_sink: IndexSink::new(),
            mmap: mmap
        }
    }

    pub fn build(&mut self, file: &File) {
        let matcher = RegexMatcher::new(r"=:\w+").unwrap();
        let mut searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(false)
            .build();
        searcher.search_file(matcher, file, &mut self.index_sink).unwrap();
    }

    pub fn index_binary(&'a mut self) -> Result<bool, Error> {
        let file_data = self.mmap.as_ref();
        for window in self.index_sink.binary_matches.windows(2) {
            let (_, offset1) = &window[0];
            let (_, offset2) = &window[1];
            let data_slice = &file_data[*offset1 as usize..*offset2 as usize];
            let mut lines = std::str::from_utf8(data_slice).unwrap().lines();
            
            if let Some((_, part2)) = lines.next().unwrap().split_once(':') {
                if let Some((hex_size, data)) = lines.next().unwrap().split_once(':') {
                    let size = usize::from_str_radix(hex_size, 16).unwrap();  
                    self.binary_index.binary_refs.insert(part2.to_string(), 
                        BinaryRef {
                            ref_name: part2.to_string(),
                            binary_data: BinaryData {
                                size,
                                data: data,
                            }
                        }
                    );
                    self.binary_index.total_size += size;
                }
            }
        }  
        Ok(true)
    }
}