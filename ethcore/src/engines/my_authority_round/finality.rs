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

//! Finality proof generation and checking.

use std::collections::{VecDeque};
use std::collections::hash_map::{HashMap, Entry};

use ethereum_types::{H256, Address};

use engines::validator_set::SimpleList;

/// Error indicating unknown validator.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UnknownValidator;

/// Rolling finality checker for authority round consensus.
/// Stores a chain of unfinalized hashes that can be pushed onto.
pub struct RollingFinality {
	headers: VecDeque<(H256, Vec<Address>)>,
	signers: SimpleList,
	sign_count: HashMap<Address, usize>,
	last_pushed: Option<H256>,
}

impl RollingFinality {
	/// Create a blank finality checker under the given validator set.
	pub fn blank(signers: Vec<Address>) -> Self {
		RollingFinality {
			headers: VecDeque::new(),
			signers: SimpleList::new(signers),
			sign_count: HashMap::new(),
			last_pushed: None,
		}
	}

	/// Extract unfinalized subchain from ancestry iterator.
	/// Clears the current subchain.
	///
	/// Fails if any provided signature isn't part of the signers set.
	pub fn build_ancestry_subchain<I>(&mut self, iterable: I) -> Result<(), UnknownValidator>
		where I: IntoIterator<Item=(H256, Vec<Address>)>
	{
		self.clear();
		for (hash, signers) in iterable {
			if signers.iter().any(|s| !self.signers.contains(s)) { return Err(UnknownValidator) }
			if self.last_pushed.is_none() { self.last_pushed = Some(hash) }

			// break when we've got our first finalized block.
			{
				let current_signed = self.sign_count.len();

				let new_signers = signers.iter().filter(|s| !self.sign_count.contains_key(s)).count();
				let would_be_finalized = (current_signed + new_signers) * 2 > self.signers.len();

				if would_be_finalized {
					trace!(target: "finality", "Encountered already finalized block {}", hash);
					break
				}

				for signer in signers.iter() {
					*self.sign_count.entry(*signer).or_insert(0) += 1;
				}
			}

			self.headers.push_front((hash, signers));
		}

		trace!(target: "finality", "Rolling finality state: {:?}", self.headers);
		Ok(())
	}

	/// Clear the finality status, but keeps the validator set.
	pub fn clear(&mut self) {
		self.headers.clear();
		self.sign_count.clear();
		self.last_pushed = None;
	}

	/// Returns the last pushed hash.
	pub fn subchain_head(&self) -> Option<H256> {
		self.last_pushed
	}

	/// Get an iterator over stored hashes in order.
	#[cfg(test)]
	pub fn unfinalized_hashes(&self) -> impl Iterator<Item=&H256> {
		self.headers.iter().map(|(h, _)| h)
	}

	/// Get the validator set.
	pub fn validators(&self) -> &SimpleList { &self.signers }

	/// Push a hash onto the rolling finality checker (implying `subchain_head` == head.parent)
	///
	/// Fails if `signer` isn't a member of the active validator set.
	/// Returns a list of all newly finalized headers.
	// TODO: optimize with smallvec.
	pub fn push_hash(&mut self, head: H256, signers: Vec<Address>) -> Result<Vec<H256>, UnknownValidator> {
		if signers.iter().any(|s| !self.signers.contains(s)) { return Err(UnknownValidator) }

		for signer in signers.iter() {
			*self.sign_count.entry(*signer).or_insert(0) += 1;
		}

		self.headers.push_back((head, signers));

		let mut newly_finalized = Vec::new();

		while self.sign_count.len() * 2 > self.signers.len() {
			let (hash, signers) = self.headers.pop_front()
				.expect("headers length always greater than sign count length; qed");

			newly_finalized.push(hash);

			for signer in signers {
				match self.sign_count.entry(signer) {
					Entry::Occupied(mut entry) => {
						// decrement count for this signer and purge on zero.
						*entry.get_mut() -= 1;

						if *entry.get() == 0 {
							entry.remove();
						}
					}
					Entry::Vacant(_) => panic!("all hashes in `header` should have entries in `sign_count` for their signers; qed"),
				}
			}
		}

		trace!(target: "finality", "Blocks finalized by {:?}: {:?}", head, newly_finalized);

		self.last_pushed = Some(head);
		Ok(newly_finalized)
	}
}
