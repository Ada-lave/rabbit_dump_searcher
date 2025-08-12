use std::{fs::{self, File}, io::Write};

use memmap2::Mmap;

mod indices;

const FILE_NAME: &str = "sample_data/2025_07_22_pr35_erl_crash.dump";

fn main() {
    let file = File::open(FILE_NAME).unwrap();
    let mmap = unsafe {
        Mmap::map(&file).unwrap()
    };
    let file_data = mmap.as_ref();
    let mut binary_index = indices::binary::BinaryIndex::new(file_data);
    let mut content_index = indices::content::ContentIndex::new(file_data);
    println!("Start building binary index");
    binary_index.build();
    println!("Start building content index");
    content_index.build();
    println!("Builded binary_index count: {}",  binary_index.index_sink.matches.len());
    println!("Builded content_index count: {}", content_index.index_sink.matches.len());

    let res = binary_index.index().unwrap();
    let messages = content_index.get_messages(&binary_index.binary_refs);
    match messages {
        Ok(messages) => {
            let mut n = 0;
            for mes in &messages {
                let mut file: File = fs::File::create(format!("out/out{}.txt", n)).unwrap();
                file.write_all(mes.as_bytes());
                n += 1;
            }
        }
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn on_small_data() {
    let file = File::open("sample_data/content.txt").unwrap();
    let mmap = unsafe {
        Mmap::map(&file).unwrap()
    };
    let file_data = mmap.as_ref();
    let mut binary_index = indices::binary::BinaryIndex::new(file_data);
    let mut content_index = indices::content::ContentIndex::new(file_data);
    binary_index.build();
    content_index.build();
    println!("Builded binary_index count: {}",  binary_index.index_sink.matches.len());
    println!("Builded content_index count: {}", content_index.index_sink.matches.len());

    let res = binary_index.index().unwrap();
    let messages = content_index.get_messages(&binary_index.binary_refs);
   match messages {
        Ok(messages) => {
            let mut n = 0;
            for mes in &messages {
                let mut file: File = fs::File::create(format!("out/out{}.txt", n)).unwrap();
                file.write_all(mes.as_bytes());
                n += 1;
            }
        }
        Err(e) => panic!("{}", e)
    }
}
