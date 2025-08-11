use std::collections::HashMap;


pub struct ProcHeapRef {
    ref_name: String,
    pub data: String
}

pub struct ProcHeapIndex {
    heap_refs: HashMap<String, ProcHeapRef>,
}

impl ProcHeapIndex {
     pub fn new() -> Self {
        Self {heap_refs: HashMap::new()}
    }
}
