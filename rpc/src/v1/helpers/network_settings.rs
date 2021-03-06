// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.
//! Structure to hold network settings configured from CLI

/// Networking & RPC settings
#[derive(Debug, PartialEq, Clone)]
pub struct NetworkSettings {
	/// Node name
	pub name: String,
	/// Name of the chain we are connected to
	pub chain: String,
	/// Networking port
	pub network_port: u16,
	/// Is JSON-RPC server enabled?
	pub rpc_enabled: bool,
	/// Interface that JSON-RPC listens on
	pub rpc_interface: String,
	/// Port for JSON-RPC server
	pub rpc_port: u16,
}

impl Default for NetworkSettings {
	fn default() -> Self {
		NetworkSettings {
			name: "".into(),
			chain: "foundation".into(),
			network_port: 30303,
			rpc_enabled: true,
			rpc_interface: "local".into(),
			rpc_port: 8545
		}
	}
}
