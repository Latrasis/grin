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

//! P2P Tokio based Protocol

use std::io;

use futures::*;
use futures::stream::*;
use futures::future::*;

use tokio_io::*;
use tokio_proto::multiplex::ServerProto;
use bytes::{BytesMut, BigEndian, BufMut, Buf, IntoBuf}; 

use core::core::{Block, Transaction};
use core::core::hash::Hash;
use core::core::target::Difficulty;
use core::ser;
use msg::*;
use types::*;

use codec::{MsgBody, MsgCodec};

/// Grin Peer Protocol
#[derive(Clone)]
struct PeerProto {
    capability: Capabilities,
    total_difficulty: Difficulty,
    version: u32,
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for PeerProto {
    type Request = MsgBody;
    type Response = MsgBody;

    // `Framed<T, LineCodec>` is the return value
    // of `io.framed(LineCodec)`
    type Transport = codec::Framed<T, MsgCodec>;
    type BindTransport = Box<Future<Item = Self::Transport, Error = io::Error>>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {

        // Construct MsgBody Transport
        let transport = io.framed(MsgCodec);
        let config = (*self).clone();
        // Get Underlying Tcp Connection
        // let con = transport.get_ref();

        Box::new(transport.into_future()
            // If the transport errors out, we don't care about
            // the transport anymore, so just keep the error
            .map_err(|(e, _)| e)
            .and_then(move |(payload, transport)| {
                // A line has been received, check to see if it
                // is the handshake
                match payload {
                    Some((id, MsgBody::Hand(ref hand))) if hand.version == config.version => {
                        println!("SERVER: received client handshake");
                        // All good, keep peer info
                        // TODO: Store Somewhere?
                        let peer_info = PeerInfo {
                            capabilities: hand.capabilities,
                            user_agent: hand.user_agent.clone(),
                            addr: hand.sender_addr.0,
                            version: hand.version,
                            total_difficulty: hand.total_difficulty.clone(),
                        };
                        // send our reply with our info
                        let shake = Shake {
                            version: PROTOCOL_VERSION,
                            capabilities: config.capability,
                            total_difficulty: config.total_difficulty,
                            user_agent: USER_AGENT.to_string(),
                        };

                        let ret = transport.send((id, MsgBody::Shake(shake)));
                        Box::new(ret) as Self::BindTransport
                    }
                    _ => {
                        // The client sent an unexpected handshake,
                        // error out the connection
                        let err = io::Error::new(io::ErrorKind::Other, "invalid handshake");
                        let ret = future::err(err);
                        Box::new(ret) as Self::BindTransport
                    }
                }
            }))
    }
}