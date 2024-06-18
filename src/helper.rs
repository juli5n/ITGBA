// This module contains miscellaneous helper utilities shared by the rest of the crate

 #![feature(lazy_cell)]
use std::collections::HashSet;
use std::sync::LazyLock;
use std::cell::RefCell;

pub trait ImmediateToStr {
    fn imm_to_str(&self) -> &str;
}

impl ImmediateToStr for std::path::Path {
    fn imm_to_str(&self) -> &str {
        self.as_os_str().to_str().unwrap()
    }
}


pub fn print_warning(str: &str) {
    println!("Warning: {}", str);
}
pub fn print_warning_once(str: &str) {
    static mut printed_warnings: LazyLock<RefCell<HashSet<String>>> = LazyLock::new(|| RefCell::new(HashSet::new()));

    let already_printed = unsafe {(*printed_warnings).borrow().contains(str)};
    if (!already_printed){
        println!("Warning: {}", str);
        unsafe {(*printed_warnings).borrow_mut().insert(String::from(str))};
    } 
}