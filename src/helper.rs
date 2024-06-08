// This module contains miscellaneous helper utilities shared by the rest of the crate

pub trait ImmediateToStr {
    fn imm_to_str(&self) -> &str;
}

impl ImmediateToStr for std::path::Path {
    fn imm_to_str(&self) -> &str {
        self.as_os_str().to_str().unwrap()
    }
}