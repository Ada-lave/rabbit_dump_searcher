use std::{fs::File};

use memmap2::Mmap;

mod indices;

const FILE_NAME: &str = "sample_data/2025_07_22_pr35_erl_crash.dump";

fn main() {
    let file = File::open(FILE_NAME).unwrap();
    let mmap = unsafe {
        Mmap::map(&file).unwrap()
    };
    let mut index = indices::global::GlobalIndex::new(&mmap);
    index.build();
    let binary_matches_count = index.index_sink.binary_matches.len();
    println!("Builded total found binary data count: {}", binary_matches_count);
    let res = index.index_binary().unwrap();
    index.show_first_n(10);
    index.dump_messages();
}
