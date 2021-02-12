#![cfg(feature = "unstable")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg(unstable)))]

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub mod decoder;
pub mod elf2table;
pub mod log;
