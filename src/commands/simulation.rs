use std::path::PathBuf;

use pallet_election_provider_multi_phase::{RawSolution};
use serde::{Serialize, Deserialize};

use crate::{
	client::Client, epm::{self, load_snapshot, mine_dry_solution, mine_solution, MinedSolution, RoundSnapshot, Snapshot, TrimmedVoters}, error::Error, helpers::storage_at, opt::Solver, prelude::*,
	signer::Signer, static_types,
};
use clap::Parser;
use codec::Encode;

#[derive(Debug, Clone, Parser)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SimulationConfig {
	/// The block hash at which scraping happens. If none is provided, the latest head is used.
	#[clap(long)]
	pub at: Option<Hash>,

	/// The solver algorithm to use.
	#[clap(subcommand)]
	pub solver: Solver,

    #[clap(long, short)]
    pub snapshot_path: PathBuf,

    #[clap(long, short)]
    pub desired_targets: u32,
}

pub async fn simulation_cmd<T>(client: Client, config: SimulationConfig) -> Result<(), Error>
where
	T: MinerConfig<AccountId = AccountId, MaxVotesPerVoter = static_types::MaxVotesPerVoter>
		+ Send
		+ Sync
		+ 'static,
	T::Solution: Send,
{
    let desired_targets = config.desired_targets;

    let content = std::fs::read_to_string(config.snapshot_path)?;
    let snap: Snapshot = serde_json::from_str(&content).expect("Not a valid JSON file");

    // print!("{:?}", snapshot);

	let storage = storage_at(config.at, client.chain_api()).await?;
	let round = storage
		.fetch_or_default(&runtime::storage().election_provider_multi_phase().round())
		.await?;
	let minimum_untrusted_score = storage
		.fetch(&runtime::storage().election_provider_multi_phase().minimum_untrusted_score())
		.await?
		.map(|score| score.0);

	let miner_solution = mine_dry_solution::<T>(
		config.solver.clone(),
		snap,
		desired_targets,
		round,
		minimum_untrusted_score
	).await?;
    
    let solution = miner_solution.solution();
	let score = miner_solution.score();
	// println!("{:?}", solution);
	let raw_solution = RawSolution { solution, score, round };
	

	log::info!(
		target: LOG_TARGET,
		"solution score {:?} / length {:?}",
		score,
		raw_solution.encode().len(),
	);
	Ok(())
}
