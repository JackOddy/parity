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

//! Ethcore rpc.
#![warn(missing_docs)]
#![cfg_attr(feature="nightly", feature(plugin))]
#![cfg_attr(feature="nightly", plugin(clippy))]

extern crate futures;
extern crate order_stat;
extern crate rustc_serialize;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate transient_hashmap;

extern crate jsonrpc_core;
extern crate jsonrpc_http_server as http;
extern crate jsonrpc_minihttp_server as minihttp;
extern crate jsonrpc_ipc_server as ipc;

extern crate ethash;
extern crate ethcore;
extern crate ethcore_io as io;
extern crate ethcore_ipc;
extern crate ethcore_light as light;
extern crate ethcrypto as crypto;
extern crate ethkey;
extern crate ethstore;
extern crate ethsync;
extern crate ethcore_logger;
extern crate fetch;
extern crate parity_reactor;
extern crate parity_updater as updater;
extern crate rlp;
extern crate stats;

#[macro_use]
extern crate log;
#[macro_use]
extern crate ethcore_util as util;
#[macro_use]
extern crate jsonrpc_macros;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate ethjson;
#[cfg(test)]
extern crate ethcore_devtools as devtools;

mod metadata;
pub mod v1;

pub use ipc::{Server as IpcServer, MetaExtractor as IpcMetaExtractor, RequestContext as IpcRequestContext};
pub use http::{
	hyper,
	RequestMiddleware, RequestMiddlewareAction,
	AccessControlAllowOrigin, Host,
};

pub use v1::{SigningQueue, SignerService, ConfirmationsQueue, NetworkSettings, Metadata, Origin, informant, dispatch};
pub use v1::block_import::is_major_importing;

use std::net::SocketAddr;
use http::tokio_core;

/// RPC HTTP Server instance
pub enum HttpServer {
	/// Fast MiniHTTP variant
	Mini(minihttp::Server),
	/// Hyper variant
	Hyper(http::Server),
}

/// RPC HTTP Server error
#[derive(Debug)]
pub enum HttpServerError {
	/// IO error
	Io(::std::io::Error),
	/// Other hyper error
	Hyper(hyper::Error),
}

impl From<http::Error> for HttpServerError {
	fn from(e: http::Error) -> Self {
		use self::HttpServerError::*;
		match e {
			http::Error::Io(io) => Io(io),
			http::Error::Other(hyper) => Hyper(hyper),
		}
	}
}

impl From<minihttp::Error> for HttpServerError {
	fn from(e: minihttp::Error) -> Self {
		use self::HttpServerError::*;
		match e {
			minihttp::Error::Io(io) => Io(io),
		}
	}
}

/// HTTP RPC server impl-independent metadata extractor
pub trait HttpMetaExtractor: Send + Sync + 'static {
	/// Type of Metadata
	type Metadata: jsonrpc_core::Metadata;
	/// Extracts metadata from given params.
	fn read_metadata(&self, origin: String, dapps_origin: Option<String>) -> Self::Metadata;
}

/// HTTP server implementation-specific settings.
pub enum HttpSettings<R: RequestMiddleware> {
	/// Enable fast minihttp server with given number of threads.
	Threads(usize),
	/// Enable standard server with optional dapps middleware.
	Dapps(Option<R>),
}

/// Start http server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_http<M, S, H, T, R>(
	addr: &SocketAddr,
	cors_domains: http::DomainsValidation<http::AccessControlAllowOrigin>,
	allowed_hosts: http::DomainsValidation<http::Host>,
	handler: H,
	remote: tokio_core::reactor::Remote,
	extractor: T,
	settings: HttpSettings<R>,
) -> Result<HttpServer, HttpServerError> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: HttpMetaExtractor<Metadata=M>,
	R: RequestMiddleware,
{
	Ok(match settings {
		HttpSettings::Dapps(middleware) => {
			let mut builder = http::ServerBuilder::new(handler)
				.event_loop_remote(remote)
				.meta_extractor(metadata::HyperMetaExtractor::new(extractor))
				.cors(cors_domains.into())
				.allowed_hosts(allowed_hosts.into());

			if let Some(dapps) = middleware {
				builder = builder.request_middleware(dapps)
			}
			builder.start_http(addr)
				.map(HttpServer::Hyper)?
		},
		HttpSettings::Threads(threads) => {
			minihttp::ServerBuilder::new(handler)
				.threads(threads)
				.meta_extractor(metadata::MiniMetaExtractor::new(extractor))
				.cors(cors_domains.into())
				.allowed_hosts(allowed_hosts.into())
				.start_http(addr)
				.map(HttpServer::Mini)?
		},
	})
}

/// Start ipc server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_ipc<M, S, H, T>(
	addr: &str,
	handler: H,
	remote: tokio_core::reactor::Remote,
	extractor: T,
) -> ::std::io::Result<ipc::Server> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: IpcMetaExtractor<M>,
{
	ipc::ServerBuilder::new(handler)
		.event_loop_remote(remote)
		.session_metadata_extractor(extractor)
		.start(addr)
}
