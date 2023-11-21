// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! The bulk (parachain slot auction) blockspace assignment provider.
//! This provider is tightly coupled with the configuration and paras modules.

use scale_info::TypeInfo;

use frame_system::pallet_prelude::BlockNumberFor;
use primitives::{CoreIndex, Id as ParaId};
use sp_runtime::codec::{Decode, Encode};

use crate::{
	configuration, paras,
	scheduler::common::{
		Assignment, AssignmentProvider, AssignmentProviderConfig, FixedAssignmentProvider,
		V0Assignment,
	},
};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + configuration::Config + paras::Config {}
}

#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone)]
pub struct ParachainsAssignment {
	pub para_id: ParaId,
}

impl ParachainsAssignment {
	fn new(para_id: ParaId) -> Self {
		Self { para_id }
	}

	pub(crate) fn from_v0_assignment(v0: V0Assignment) -> Self {
		Self { para_id: v0.para_id }
	}
}

impl Assignment for ParachainsAssignment {
	fn para_id(&self) -> ParaId {
		self.para_id
	}
}

impl<T: Config> AssignmentProvider<BlockNumberFor<T>> for Pallet<T> {
	type AssignmentType = ParachainsAssignment;

	fn pop_assignment_for_core(core_idx: CoreIndex) -> Option<Self::AssignmentType> {
		<paras::Pallet<T>>::parachains()
			.get(core_idx.0 as usize)
			.copied()
			.map(|para_id| ParachainsAssignment::new(para_id))
	}

	fn report_processed(_: Self::AssignmentType) {}

	/// Bulk assignment has no need to push the assignment back on a session change,
	/// this is a no-op in the case of a bulk assignment slot.
	fn push_back_assignment(_: Self::AssignmentType) {}

	fn get_provider_config(_core_idx: CoreIndex) -> AssignmentProviderConfig<BlockNumberFor<T>> {
		AssignmentProviderConfig {
			// The next assignment already goes to the same [`ParaId`], no timeout tracking needed.
			max_availability_timeouts: 0,
			// The next assignment already goes to the same [`ParaId`], this can be any number
			// that's high enough to clear the time it takes to clear backing/availability.
			ttl: 10u32.into(),
		}
	}

	#[cfg(any(feature = "runtime-benchmarks", test))]
	fn get_mock_assignment(_: CoreIndex, para_id: ParaId) -> Self::AssignmentType {
		ParachainsAssignment { para_id }
	}
}

impl<T: Config> FixedAssignmentProvider<BlockNumberFor<T>> for Pallet<T> {
	fn session_core_count() -> u32 {
		paras::Parachains::<T>::decode_len().unwrap_or(0) as u32
	}
}
