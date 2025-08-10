use std::fs::File;
use std::io::Error;
use std::os::unix::fs::FileExt;
use std::{fs, io};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use grep_regex::RegexMatcher;
use grep_searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use grep_searcher::{Searcher};

pub struct BinaryData {
    pub size: usize,
    pub data: Vec<u8>
}

pub struct RabbitFraming {
    pub ref_name: String,
    pub message_parts: String 
}

pub struct BinaryRef {
    ref_name: String,
    pub binary_data: BinaryData
}

pub struct IndexSink {
    pub matches: Vec<(Option<String>, u64)>,
}

impl IndexSink {
    fn new() -> Self {
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
        if match_bytes.starts_with(b"=") {
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
        }
        Ok(true)
    }
}

pub struct BinaryIndex {
    pub index_sink: IndexSink,
    total_size: usize,
    pub records: Vec<BinaryRef>
}

impl BinaryIndex {
    pub fn new() -> Self {
        Self { total_size:0, index_sink: IndexSink::new(), records: Vec::new() }
    }

    pub fn build(&mut self, file: &File) {
        let matcher = RegexMatcher::new(r"=binary:\w+").unwrap();
        let mut searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(false)
            .build();
        searcher.search_file(matcher, file, &mut self.index_sink).unwrap();
    }
    pub fn index(&mut self, file: &File) -> Result<bool, Error> {
        let mut find_duplicated = 0;
        for window in self.index_sink.matches.windows(2) {
            let (_, offset1) = &window[0];
            let (_, offset2) = &window[1];
            let mut buffer = vec![0; (offset2 - offset1) as usize];
            let _ = file.read_exact_at(&mut buffer, *offset1);
            let content = String::from_utf8_lossy(&buffer);

            let mut lines = content.lines();
            
            if let Some((_, part2)) = lines.next().unwrap().split_once(':') {
                if let Some((hex_size, data)) = lines.next().unwrap().split_once(':') {
                    let size = usize::from_str_radix(hex_size, 16).unwrap();  
                    self.records.push(BinaryRef {
                        ref_name: part2.to_string(),
                        binary_data: BinaryData {
                            size,
                            data: "sample".as_bytes().to_vec(), // Пока заглушка
                        }
                    });
                    if size == 131064 && find_duplicated <= 10 {
                        let raw_json = BASE64_STANDARD.decode(data).unwrap();
                        fs::write(format!("{}.json", part2,), raw_json);
                        find_duplicated += 1;
                    }
                    self.total_size += 1
                }
            }
        }  
        println!("{}",find_duplicated);
        Ok(true)
    }

    pub fn sort_by_size(&mut self) {
        self.records.sort_by_key(|record| record.binary_data.size);
        self.records.reverse();
    }
}