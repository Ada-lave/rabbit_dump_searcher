use std::{fs::File};

mod types;

const FILE_NAME: &str = "sample_data/2025_07_22_pr35_erl_crash.dump";

fn main() { 
    let mut index = types::BinaryIndex::new();
    let file = File::open(FILE_NAME).unwrap();
    index.build(&file);
    println!("Builded total found binary data count: {}", index.index_sink.matches.len());
    index.index(&file).unwrap();
    index.sort_by_size();
    for i in 0..10000 {
        println!("size {}", index.records[i].binary_data.size);
        // println!("size {} - {}", index.records[i].binary_data.size, String::from_utf8_lossy(&index.records[i].binary_data.data));
    } 
}
