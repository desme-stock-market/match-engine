mod engine;
mod matcher;
mod model;
mod policies;
mod storage;

use anyhow::Result;

use crate::engine::MatchEngine;

#[tokio::main]
async fn main() -> Result<()> {
    let match_engine = MatchEngine::new();
    Ok(())
}
