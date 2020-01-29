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

//! Authority params deserialization.

use hash::Address;
use uint::Uint;
use bytes::Bytes;
use super::ValidatorSet;

/// Authority params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MAuraParams {
	/// Block duration, in seconds.
	pub step_duration: Uint,
	/// Valid authorities
	pub validators: ValidatorSet,
	/// Starting step. Determined automatically if not specified.
	/// To be used for testing only.
	pub start_step: Option<Uint>,
	/// Block at which score validation should start.
	pub validate_score_transition: Option<Uint>,
	/// Block from which monotonic steps start.
	pub validate_step_transition: Option<Uint>,
	/// Whether transitions should be immediate.
	pub immediate_transitions: Option<bool>,
	/// Reward per block in wei.
	pub block_reward: Option<Uint>,
	/// Block at which the block reward contract should start being used.
	pub block_reward_contract_transition: Option<Uint>,
	/// Block reward contract address (setting the block reward contract
	/// overrides the static block reward definition).
	pub block_reward_contract_address: Option<Address>,
	/// Block reward code. This overrides the block reward contract address.
	pub block_reward_contract_code: Option<Bytes>,
	/// Block at which maximum uncle count should be considered.
	pub maximum_uncle_count_transition: Option<Uint>,
	/// Maximum number of accepted uncles.
	pub maximum_uncle_count: Option<Uint>,
	/// Block at which empty step messages should start.
	pub empty_steps_transition: Option<Uint>,
	/// Maximum number of accepted empty steps.
	pub maximum_empty_steps: Option<Uint>,
	/// Strict validation of empty steps transition block.
	pub strict_empty_steps_transition: Option<Uint>,
}

/// Authority engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MAura {
	/// Ethash params.
	pub params: MAuraParams,
}