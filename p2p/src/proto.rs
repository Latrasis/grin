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

use tokio_io::*;
use tokio_proto::multiplex::ServerProto;
use bytes::{BytesMut, BigEndian, BufMut, Buf, IntoBuf}; 

use core::core::{Block, Transaction};
use core::core::hash::Hash;
use core::ser;
use msg::*;

use codec::{MsgBody, MsgCodec};

/// Grin Peer Protocol
struct PeerProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for PeerProto {
    type Request = MsgBody;
    type Response = MsgBody;

    // `Framed<T, LineCodec>` is the return value
    // of `io.framed(LineCodec)`
    type Transport = codec::Framed<T, MsgCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(MsgCodec))
    }
}