use std::{fs::File};

mod indices;

const FILE_NAME: &str = "sample_data/2025_07_22_pr35_erl_crash.dump";

fn main() {
    let file = File::open(FILE_NAME).unwrap();
    let mut index = indices::global::GlobalIndex::new(&file);
    index.build(&file);
    println!("Builded total found binary data count: {}", index.index_sink.binary_matches.len());
    index.index_binary().unwrap();
    
}
