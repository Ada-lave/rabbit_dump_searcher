use std::{fs::File};
use std::io::{BufRead, BufReader, prelude::*};
use base64::read;
use base64::{prelude::BASE64_STANDARD, Engine};

use grep::matcher::Matcher;
use grep_regex::{RegexMatcher};
use grep_searcher::{sinks::UTF8, Searcher};

const DATA: &'static [u8] = b"\
=binary:c29tZSBkYXRh
98:c29tZSBkYXRh
=binary:7F910BA94220
1AC4:c29tZSB0ZXh0
";


struct BinaryData {
    size: usize,
    data: Vec<u8>
}


struct BinaryRef {
    ref_name: String,
    binary_data: BinaryData
}

struct BinaryIndex {
    raw_records: Vec<(u64, String)>,
    records: Vec<BinaryRef>
}

const FILE_NAME: &str = "erl_crash_20250105-004018.dump";

impl BinaryIndex {
    fn new() -> Self {
        Self { raw_records: Vec::new(), records: Vec::new() }
    }
    fn build(&mut self) {
        let matcher = RegexMatcher::new(r"=binary:\w+").unwrap();
        Searcher::new().search_path(&matcher, FILE_NAME, UTF8(|lnum, line| {
            let mymatch = matcher.find(line.as_bytes())?.unwrap();
            self.raw_records.push((lnum - 1, line[mymatch].to_string()));
            Ok(true)
        })).unwrap();
    }
    fn index(&mut self) {
        // Открываем файл и создаём буфер
        let file = File::open(FILE_NAME).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
         if line.unwrap() == "asdas" {
            
         }   
        }
        for (cur_line_num, value) in &self.raw_records {
            println!("Start work on line:{}", cur_line_num);
            

            // Читаем нужную строку по номеру: +1, потому что data всегда на следующей строке
            let data_line_index = *cur_line_num + 1;
            let data_line_opt = reader.lines()
                .enumerate()
                .skip_while(|(idx, _)| *idx as u64 != data_line_index)
                .next();

            if let Some((_, Ok(data_line))) = data_line_opt {
                if let Some((_, part2)) = value.split_once(':') {
                    if let Some((hex_size, data)) = data_line.split_once(':') {
                        let size = usize::from_str_radix(hex_size, 16).unwrap();  
                        // let decoded_data = BASE64_STANDARD.decode(data.as_bytes()).unwrap(); 
                        self.records.push(BinaryRef {
                            ref_name: part2.to_string(),
                            binary_data: BinaryData {
                                size,
                                data: "sample".as_bytes().to_vec(), // Пока заглушка
                            }
                        });
                    }
                }
            }
        }
    }

    fn sort_by_size(&mut self) {
        self.records.sort_by_key(|record| record.binary_data.size);
        self.records.reverse();
    }
}

fn main() { 

    let mut index = BinaryIndex::new();
    index.build();
    println!("Builded total found binary data count: {}", index.raw_records.len());
    index.index();
    index.sort_by_size();
    for i in 0..3 {
        println!("size {}", index.records[i].binary_data.size);
        // println!("size {} - {}", index.records[i].binary_data.size, String::from_utf8_lossy(&index.records[i].binary_data.data));
    } 
}
