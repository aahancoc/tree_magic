// Common routines for all fdo_magic parsers

pub mod builtin;

#[derive(Debug, Clone)]
pub struct MagicRule {
    pub indent_level: u32,
    pub start_off: u32,
    pub val_len: u16,
    pub val: Vec<u8>,
    pub mask: Option<Vec<u8>>,
    pub word_len: u32,
    pub region_len: u32
}

pub mod ruleset;
pub mod check;
