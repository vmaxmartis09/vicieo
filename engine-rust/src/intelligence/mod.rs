//! Track D — intelligence (products, blueprints, ideas, scripts, policy) ported from Python `socnhotv.intelligence`.
//! Data layout is identical to the Python backend (JSON files under data/projects/<id>/...).
pub mod analyzer;
pub mod channels;
pub mod chat;
pub mod compliance;
pub mod content_policy;
pub mod contracts;
pub mod ideas;
pub mod koc_styles;
pub mod node_runner;
pub mod patterns;
pub mod products;
pub mod review_flow;
pub mod script_gen;
pub mod strategy;
pub mod tiktok_product;
pub mod util;

// `sources/` lives beside this module (Track D owns both); declared here so main.rs stays untouched.
#[path = "../sources/mod.rs"]
pub mod sources;
