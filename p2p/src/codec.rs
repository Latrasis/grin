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

//! P2P Message Encoding and Decoding

use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::multiplex::*;
use bytes::{BytesMut, BufMut, Buf};

use core::core::{self, Block, BlockHeader, Transaction};
use core::core::hash::Hash;
use core::ser;
use msg::*;
use types::*;

#[derive(Clone, PartialEq)]
enum MsgBody {
	PeerError(PeerError),
	Hand(Hand),
	Shake(Shake),
	Ping,
	Pong,
	GetPeerAddrs(GetPeerAddrs),
	PeerAddrs(PeerAddrs),
	GetHeaders(Locator),
	Headers(Headers),
	GetBlock(Hash),
	Block(Block),
	Transaction(Transaction),
}

/// P2P Message Encoder and Decoder
#[derive(Default, Clone)]
struct MsgCodec;

impl Encoder for MsgCodec {
    type Item = (RequestId, MsgBody);
    type Error = ser::Error;

    fn encode(&mut self, (id, msg): Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        
		let (header, body) = match msg {
			MsgBody::Pong => (MsgHeader::new(Type::Pong, 0), vec![]),
			MsgBody::Ping => (MsgHeader::new(Type::Ping, 0), vec![]),
			MsgBody::Hand(hand) => {
				let body = ser::ser_vec(&hand)?;
				(MsgHeader::new(Type::Hand, body.len() as u64), body)
			}
			MsgBody::Shake(shake) => {
				let body = ser::ser_vec(&shake)?;
				(MsgHeader::new(Type::Shake, body.len() as u64), body)
			}
			MsgBody::GetPeerAddrs(get_peer_addrs) => {
				let body = ser::ser_vec(&get_peer_addrs)?;
				(MsgHeader::new(Type::GetPeerAddrs, body.len() as u64), body)
			}
			MsgBody::PeerAddrs(peer_addrs) => {
				let body = ser::ser_vec(&peer_addrs)?;
				(MsgHeader::new(Type::PeerAddrs, body.len() as u64), body)
			}
			MsgBody::Headers(headers) => {
				let body = ser::ser_vec(&headers)?;
				(MsgHeader::new(Type::Headers, body.len() as u64), body)
			}
			MsgBody::GetHeaders(locator) => {
				let body = ser::ser_vec(&locator)?;
				(MsgHeader::new(Type::GetHeaders, body.len() as u64), body)
			}
			MsgBody::Block(block) => {
				let body = ser::ser_vec(&block)?;
				(MsgHeader::new(Type::Block, body.len() as u64), body)
			}
			MsgBody::GetBlock(hash) => {
				let body = ser::ser_vec(&hash)?;
				(MsgHeader::new(Type::GetBlock, body.len() as u64), body)
			}
			MsgBody::Transaction(tx) => {
				let body = ser::ser_vec(&tx)?;
				(MsgHeader::new(Type::Transaction, body.len() as u64), body)
			}
			MsgBody::PeerError(err) => {
				let body = ser::ser_vec(&err)?;
				(MsgHeader::new(Type::Error, body.len() as u64), body)
			}
		};

		// Serialize MsgHeader
        let header = ser::ser_vec(&header)?;

        // Put Header and Body
        dst.reserve(header.len() + body.len());
        dst.put_slice(&header);
        dst.put_slice(&body);

		Ok(())
    }

}

impl Decoder for MsgCodec {
	type Item = (RequestId, MsgBody);
	type Error = ser::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}