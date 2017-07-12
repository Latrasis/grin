// Copyright 2016 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::{Mutex, Arc};

use futures;
use futures::Future;
use futures::stream;
use futures::sync::mpsc::UnboundedSender;
use tokio_core::net::TcpStream;

use core::core;
use core::core::hash::Hash;
use core::ser;
use conn::TimeoutConnection;
use msg::*;
use types::*;
use util::OneTime;

pub struct ProtocolV1 {
	conn: OneTime<TimeoutConnection>,

	expected_responses: Mutex<Vec<(Type, Hash)>>,
}

impl ProtocolV1 {
	pub fn new() -> ProtocolV1 {
		ProtocolV1 {
			conn: OneTime::new(),
			expected_responses: Mutex::new(vec![]),
		}
	}
}

impl Protocol for ProtocolV1 {
	/// Sets up the protocol reading, writing and closing logic.
	fn handle(&self,
	          conn: TcpStream,
	          adapter: Arc<NetAdapter>)
	          -> Box<Future<Item = (), Error = Error>> {

		let (conn, listener) = TimeoutConnection::listen(conn, move |sender, header, data| {
			let adapt = adapter.as_ref();
			handle_payload(adapt, sender, header, data).map_err(|_| ser::Error::CorruptedData)
		});

		self.conn.init(conn);

		listener
	}

	/// Bytes sent and received.
	fn transmitted_bytes(&self) -> (u64, u64) {
		self.conn.borrow().transmitted_bytes()
	}

	/// Sends a ping message to the remote peer. Will panic if handle has never
	/// been called on this protocol.
	fn send_ping(&self) -> Result<(), Error> {
		self.send_request(Type::Ping, Type::Pong, &Empty {}, None)
	}

	/// Serializes and sends a block to our remote peer
	fn send_block(&self, b: &core::Block) -> Result<(), Error> {
		self.send_msg(Type::Block, b)
	}

	/// Serializes and sends a transaction to our remote peer
	fn send_transaction(&self, tx: &core::Transaction) -> Result<(), Error> {
		self.send_msg(Type::Transaction, tx)
	}

	fn send_header_request(&self, locator: Vec<Hash>) -> Result<(), Error> {
		self.send_request(Type::GetHeaders,
		                  Type::Headers,
		                  &Locator { hashes: locator },
		                  None)
	}

	fn send_block_request(&self, h: Hash) -> Result<(), Error> {
		self.send_request(Type::GetBlock, Type::Block, &h, Some(h))
	}

	fn send_peer_request(&self, capab: Capabilities) -> Result<(), Error> {
		self.send_request(Type::GetPeerAddrs,
		                  Type::PeerAddrs,
		                  &GetPeerAddrs { capabilities: capab },
		                  None)
	}

	/// Close the connection to the remote peer
	fn close(&self) {
		// TODO some kind of shutdown signal
	}
}

impl ProtocolV1 {
	fn send_msg<W: ser::Writeable>(&self, t: Type, body: &W) -> Result<(), Error> {
		self.conn.borrow().send_msg(t, body)
	}

	fn send_request<W: ser::Writeable>(&self,
	                                   t: Type,
	                                   rt: Type,
	                                   body: &W,
	                                   expect_resp: Option<Hash>)
	                                   -> Result<(), Error> {
		self.conn.borrow().send_request(t, rt, body, expect_resp)
	}
}

fn handle_payload(adapter: &NetAdapter,
                  sender: UnboundedSender<Vec<u8>>,
                  header: MsgHeader,
                  buf: Vec<u8>)
                  -> Result<Option<Hash>, Error> {
	match header.msg_type {
		Type::Ping => {
			let data = ser::ser_vec(&MsgHeader::new(Type::Pong, 0))?;
			sender.send(data);
            Ok(None)
		}
		Type::Pong => Ok(None),
		Type::Transaction => {
			let tx = ser::deserialize::<core::Transaction>(&mut &buf[..])?;
			adapter.transaction_received(tx).and(Ok(None))

		}
		Type::GetBlock => {
			let h = ser::deserialize::<Hash>(&mut &buf[..])?;
			let bo = adapter.get_block(h);
			if let Some(b) = bo {
				// serialize and send the block over
				let mut body_data = vec![];
				try!(ser::serialize(&mut body_data, &b));
				let mut data = vec![];
				try!(ser::serialize(&mut data,
				                    &MsgHeader::new(Type::Block, body_data.len() as u64)));
				data.append(&mut body_data);
				sender.send(data);
			}
			Ok(None)
		}
		Type::Block => {
			let b = ser::deserialize::<core::Block>(&mut &buf[..])?;
			let bh = b.hash();
			adapter.block_received(b).and(Ok(Some(bh)))
		}
		Type::GetHeaders => {
			// load headers from the locator
			let loc = ser::deserialize::<Locator>(&mut &buf[..])?;

            if let Some(headers) = adapter.locate_headers(loc.hashes) {
                // serialize and send all the headers over
                let mut body_data = vec![];
                try!(ser::serialize(&mut body_data, &Headers { headers: headers }));
                let mut data = vec![];
                try!(ser::serialize(&mut data,
                                    &MsgHeader::new(Type::Headers, body_data.len() as u64)));
                data.append(&mut body_data);
                sender.send(data);
            }

			Ok(None)
		}
		Type::Headers => {
			let headers = ser::deserialize::<Headers>(&mut &buf[..])?;
			adapter.headers_received(headers.headers).and(Ok(None))
		}
		Type::GetPeerAddrs => {
			let get_peers = ser::deserialize::<GetPeerAddrs>(&mut &buf[..])?;
            
            if let Some(peer_addrs) = adapter.find_peer_addrs(get_peers.capabilities) {
                // serialize and send all the headers over
                let mut body_data = vec![];
                try!(ser::serialize(&mut body_data,
                                    &PeerAddrs {
                                        peers: peer_addrs.iter().map(|sa| SockAddr(*sa)).collect(),
                                    }));
                let mut data = vec![];
                try!(ser::serialize(&mut data,
                                    &MsgHeader::new(Type::PeerAddrs, body_data.len() as u64)));
                data.append(&mut body_data);
                sender.send(data);
            }

			Ok(None)
		}
		Type::PeerAddrs => {
			let peer_addrs = ser::deserialize::<PeerAddrs>(&mut &buf[..])?;
			adapter.peer_addrs_received(peer_addrs.peers.iter().map(|pa| pa.0).collect()).and(Ok(None))
		}
		_ => {
			debug!("unknown message type {:?}", header.msg_type);
			Ok(None)
		}
	}
}
