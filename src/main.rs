use std::{fs::File};

use memmap2::Mmap;

mod indices;

const FILE_NAME: &str = "sample_data/2025_07_22_pr35_erl_crash.dump";

fn main() {
    let file = File::open(FILE_NAME).unwrap();
    let mmap = unsafe {
        Mmap::map(&file).unwrap()
    };
    let mut binary_index = indices::binary::BinaryIndex::new(&mmap);
    let mut content_index = indices::content::ContentIndex::new(&mmap);
    binary_index.build();
    content_index.build();
    let binary_matches_count = binary_index.index_sink.matches.len();
    println!("Builded total found binary data count: {}", binary_matches_count);
    let res = binary_index.index().unwrap();

}
