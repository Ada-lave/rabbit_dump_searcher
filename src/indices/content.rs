use std::collections::HashMap;

use grep_regex::{Error, RegexMatcher};
use grep_searcher::{BinaryDetection, SearcherBuilder};
use memmap2::Mmap;

use crate::indices::binary::BinaryRef;
use crate::indices::global::IndexSink;

pub struct ContentIndex<'a> {
    pub index_sink: IndexSink,
    pub mmap: &'a Mmap

}


impl <'a>ContentIndex<'a> {
    pub fn new(mmap: &'a Mmap) -> Self {
        Self {
            mmap: mmap,
            index_sink: IndexSink::new(),
        }
    }

    pub fn build(&mut self) -> Result<(), Error> {
        let matcher = RegexMatcher::new(r"^[A-Za-z0-9]{12}:t6:A7:content*")?;
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

    pub fn get_messages(&self, binary_refs: &HashMap<String, BinaryRef>) -> Result<Vec<String>, Error> {
        let mut messages: Vec<String> = Vec::new();
        let file_data = self.mmap.as_ref();
        for window in self.index_sink.matches.windows(2) {
            let (_, offset1) = &window[0];
            let (_, offset2) = &window[1];
            let mut message_parts: Vec<String> = Vec::new();
            let data_slice = &file_data[*offset1 as usize..*offset2 as usize];
            let mut proc_heap_data = std::str::from_utf8(data_slice).unwrap();

            println!("{proc_heap_data}");
            // let mut content_data_ref = proc_heap_data.next();
            
            // loop {
            //     match content_data_ref {
            //         Some(data) => {
            //             if !data.contains(':') && data.contains("content") {
            //                 content_data_ref = proc_heap_data.next();
            //                 continue;
            //             }
            //             println!("{}", data);
            //             let (_, list) = data.split_once(':').unwrap();
            //             let (young_heap_reff, next_data_reff) = list.split_once(':').unwrap();
            //             message_parts.push(self.find_message_part(binary_refs, young_heap_reff.to_string()));
            //             if next_data_reff != "N" {
            //                 content_data_ref = proc_heap_data.next();
            //             }
            //         }
            //         None => break
            //     }
            // }
            // messages.push(message_parts.join(""));
        }
        if let Some(last_match) = self.index_sink.matches.last() {
            let (_, offset1) = last_match;
            let mut message_parts: Vec<String> = Vec::new();
            let data_slice = &file_data[*offset1 as usize..*&file_data.len() as usize];
            let mut proc_heap_data = std::str::from_utf8(data_slice).unwrap();

            println!("{proc_heap_data}");
        }
        Ok(messages)
    }

    fn find_message_part(&self, binary_refs: &HashMap<String, BinaryRef>, key: String) -> String {
        let bin_ref = binary_refs.get(&key);
        return bin_ref.unwrap().binary_data.data.to_string();
    }
}

#[test]
fn build_index() {
    use std::{fs::File};
    const FILE_NAME: &str = "sample_data/content.txt";
    let file = File::open(FILE_NAME).unwrap();
    let mmap = unsafe {
        Mmap::map(&file).unwrap()
    };
    let mut content_index = ContentIndex::new(&mmap);
    content_index.build().unwrap();
    content_index.get_messages(&HashMap::new()).unwrap();
}