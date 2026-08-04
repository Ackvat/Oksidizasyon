//#![no_std]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Parser for custom compile settings.
macro_rules! parse_env_numeric {
    ($name:ident, $ty:ty, $env_var:expr, $default_str:expr) => {
        pub(crate) const $name: $ty = {
            const STR_VAL: &str = match option_env!($env_var) {
                Some(val) => val,
                None => $default_str,
            };

            let bytes = STR_VAL.as_bytes();
            if bytes.is_empty() { 
                panic!(concat!("Compile setting error: ", $env_var, " cannot be empty!")); 
            }
            
            let mut value: $ty = 0;
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if b < b'0' || b > b'9' { 
                    panic!(concat!("Compile setting error: ", $env_var, " must be a valid integer!")); 
                }
                value = value * 10 + (b - b'0') as $ty;
                i += 1;
            }
            value
        };
    };
}



//parse_env_numeric!(NODE_SIZE, usize, "NODE_SIZE", "8");
//parse_env_numeric!(MAX_OBJECTS, usize, "MAX_OBJECTS", "128");



// Base Module, contains the base abstraction objects and their tools.
pub mod base;

// Maths Module, contains the commonly used math objects and applications.
pub mod maths;

// Most needed utilities are in this module.
pub mod utils;



#[cfg(feature = "emd")]
pub unsafe fn init_emd_heap(start_addr: usize, size: usize, allocator: &embedded_alloc::LlffHeap) {
    allocator.init(start_addr, size);
}
