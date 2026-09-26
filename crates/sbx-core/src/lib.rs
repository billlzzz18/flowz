//! sbx-core: ชั้นกลางที่ทำให้ทุก backend มีหน้าตาเดียวกัน
//! หน่วยพื้นฐานคือ `spawn(cmd) -> Process{stdin, stdout, stderr, wait}`

pub mod backend;
pub mod backends;
pub mod config;

pub use backend::{Process, ProcessSpec, Sandbox};
pub use config::Config;
