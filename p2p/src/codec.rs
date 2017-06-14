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

use std::io;

use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::multiplex::*;
use bytes::{BytesMut, BigEndian, BufMut, Buf, IntoBuf}; 

use core::core::{Block, Transaction};
use core::core::hash::Hash;
use core::ser;
use msg::*;

const MSG_HEADER_SIZE: usize = 11;

#[derive(Clone, PartialEq)]
pub enum MsgBody {
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
pub struct MsgCodec;

impl Encoder for MsgCodec {
    type Item = (RequestId, MsgBody);
    type Error = io::Error;

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

        // Put Header, Id and Body
        dst.reserve(MSG_HEADER_SIZE + 8 + body.len());

        dst.put_slice(&header);
        dst.put_u64::<BigEndian>(id);
        dst.put_slice(&body);

		Ok(())
    }

}

impl Decoder for MsgCodec {
	type Item = (RequestId, MsgBody);
	type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Create Temporary Buffer
		let ref mut temp_src = src.clone();
		
		// Check Minimum Size
		if temp_src.len() < MSG_HEADER_SIZE + 8 {
			return Ok(None);
		}

        // Get MsgHeader
		let buf = temp_src.split_to(MSG_HEADER_SIZE).into_buf();
        let header = ser::deserialize::<MsgHeader>(&mut buf.reader())?;
        
        // Get Id
        let mut buf = temp_src.split_to(8).into_buf();
        let id = buf.get_u64::<BigEndian>();

		// Ensure sufficient data
		if temp_src.len() < header.msg_len as usize {
			return Ok(None);
		}

        // Extract Reader
        let ref mut read = temp_src.split_to(header.msg_len as usize).into_buf().reader();

		// Attempt message body decode
		let decoded_msg = match header.msg_type {
			Type::Ping => MsgBody::Ping,
			Type::Pong => MsgBody::Pong,
			Type::Hand => {
				let hand = ser::deserialize::<Hand>(read)?;
				MsgBody::Hand(hand)
			}
			Type::Shake => {
				let shake = ser::deserialize::<Shake>(read)?;
				MsgBody::Shake(shake)
			}
			Type::GetPeerAddrs => {
				let get_peer_addrs = ser::deserialize::<GetPeerAddrs>(read)?;
				MsgBody::GetPeerAddrs(get_peer_addrs)
			}
			Type::PeerAddrs => {
				let peer_addrs = ser::deserialize::<PeerAddrs>(read)?;
				MsgBody::PeerAddrs(peer_addrs)
			}
			Type::Headers => {
				let headers = ser::deserialize::<Headers>(read)?;
				MsgBody::Headers(headers)
			}
			Type::GetHeaders => {
				let locator = ser::deserialize::<Locator>(read)?;
				MsgBody::GetHeaders(locator)
			}
			Type::Block => {
				let block = ser::deserialize::<Block>(read)?;
				MsgBody::Block(block)
			}
			Type::GetBlock => {
				let hash = ser::deserialize::<Hash>(read)?;
				MsgBody::GetBlock(hash)
			}
			Type::Transaction => {
				let transaction = ser::deserialize::<Transaction>(read)?;
				MsgBody::Transaction(transaction)
			}
			Type::Error => {
				let err = ser::deserialize::<PeerError>(read)?;
				MsgBody::PeerError(err)
			}
		};

		// If succesfull truncate src by bytes read from temp_src;
		let diff = src.len() - temp_src.len();
		src.split_to(diff);

		Ok(Some((id, decoded_msg)))
	}
}