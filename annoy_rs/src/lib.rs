#![feature(duration_as_u128)]
#![feature(trait_alias)]
extern crate libc;
#[cfg(test)]
extern crate rand;

pub mod annoy;
pub mod err;
pub mod idmapping;
mod vector;

mod native {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
