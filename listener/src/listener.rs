// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::{Arc, Weak};

use common_types::filter::Filter;
use ethcore::client::{BlockId, BlockChainClient, ChainNotify, NewBlocks};
use itertools::Itertools;

use_contract!(mycontract, "res/my_contract.json");

/// Service for checking for updates and determining whether we can achieve consensus.
pub struct Listener {
	// Useful environmental stuff.
	client: Weak<BlockChainClient>
}

impl Listener {
	/// `Updater` constructor
	pub fn new(
		client: &Weak<BlockChainClient>
	) -> Arc<Listener> {
		let r = Arc::new(Listener {
			client: client.clone(),
		});
		r 
	}
}

impl Listener {
	/// Set a closure to call when we want to restart the client
	fn do_things(&self) -> Option<bool> {
		let current_block_number = self.client.upgrade().map_or(0, |c| c.block_number(BlockId::Latest).unwrap_or(0));
		let client = self.client.upgrade();
		if !client.is_some() {
			info!("Client not found");
			return None;
		}
		let client = client.unwrap();
		let address = client.registry_address("twitter".into(), BlockId::Latest);
		if !address.is_some() {
			info!("Address not found");
			return None;
		}
		let address = address.unwrap();

		let topics = mycontract::events::tweet::filter();
		let topics = vec![topics.topic0, topics.topic1, topics.topic2, topics.topic3];
		let topics = topics.into_iter().map(Into::into).map(Some).collect();

		let filter = Filter {
			from_block: BlockId::Number(current_block_number),
			to_block: BlockId::Latest,
			address: Some(vec![address]),
			topics,
			limit: None,
		};
		info!("Start to read logs");

		client.logs(filter)
			.unwrap_or_default()
			.iter().foreach(|log| {
				let event = mycontract::events::tweet::parse_log((log.topics.clone(), log.data.clone()).into()).ok();
				if event.is_some() {
					let event = event.unwrap();
					info!("Message: Address {} says {}", event.user, event.message);
				}
			});
		return Some(true);
	}

	fn poll(&self) {
		let _ = self.do_things();
	}
}

impl ChainNotify for Listener {
	fn new_blocks(&self, new_blocks: NewBlocks) {
		//if new_blocks.has_more_blocks_to_import { return }
		self.poll();
	}
}
