// Copyright 2016 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Main crate putting together all the other crates that compose Grin into a
//! binary.

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
extern crate futures_cpupool as cpupool;
extern crate hyper;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate tokio_core;
extern crate tokio_timer;
#[macro_use]
extern crate lazy_static;
extern crate itertools;

extern crate grin_api as api;
extern crate grin_chain as chain;
#[macro_use]
extern crate grin_core as core;
extern crate grin_p2p as p2p;
extern crate grin_pool as pool;
extern crate grin_store as store;
extern crate grin_util as util;
extern crate grin_wallet as wallet;
extern crate secp256k1zkp as secp;

extern crate cuckoo_miner;

mod adapters;
mod miner;
mod plugin;
mod server;
mod seed;
mod sync;
mod types;

pub use server::{Server};
pub use types::{ServerConfig, MinerConfig, Seeding, ServerStats};
pub use plugin::PluginMiner;
