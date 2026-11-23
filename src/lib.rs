//! Client Liabilities API: CRUD for a client's liabilities, with staged client changes that an
//! advisor reviews. See the README for the staging rules.

pub mod api;
pub mod codec;
pub mod config;
pub mod db;
pub mod domain;
pub mod dto;
pub mod error;
pub mod seed;
pub mod service;
