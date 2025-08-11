use std::collections::HashMap;
pub struct BinaryData<'a> {
    pub size: usize,
    pub data: &'a str
}

pub struct RabbitFraming {
    pub ref_name: String,
    pub message_parts: String 
}

pub struct BinaryRef<'a> {
    pub ref_name: String,
    pub binary_data: BinaryData<'a>
}

pub struct BinaryIndex<'a> {
    pub total_size: usize,
    pub binary_refs: HashMap<String, BinaryRef<'a>>
}

impl <'a> BinaryIndex <'a> {
    pub fn new() -> Self {
        Self {total_size:0, binary_refs: HashMap::new() }
    }

    pub fn get_ref_by_name(&self, ref_name: &str) -> Option<&BinaryRef<'_>> {
        return self.binary_refs.get(ref_name);
    }
}