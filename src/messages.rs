// This file is part of paperd, the PaperMC server daemon
// Copyright (C) 2019 Kyle Wood (DemonWav)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, version 3 only.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::protocol::ProtocolVersionMessage;
use crate::restart::RestartMessage;
use crate::send::SendCommandMessage;
use crate::status::StatusMessage;
use crate::stop::StopMessage;
use crate::timings::TimingsMessage;
use serde::Deserialize;
#[cfg(feature = "console")]
use {
    crate::console::ConsoleStatusMessage, crate::console::EndLogsListenerMessage,
    crate::console::LogsMessage, crate::console::TabCompleteMessage,
};

pub trait MessageHandler {
    fn type_id() -> i64;
}

// Special error message
#[derive(Deserialize)]
pub struct ServerErrorMessage {
    #[serde(rename = "error")]
    pub error: Option<String>,
    #[serde(rename = "shutdown")]
    pub is_shutdown: bool,
}

macro_rules! message_version {
    ($ver:expr, $type:ty) => {
        impl MessageHandler for $type {
            fn type_id() -> i64 {
                return $ver;
            }
        }
    };
    ($ver:expr, $type:ty, console) => {
        #[cfg(feature = "console")]
        message_version!($ver, $type);
    };
}

message_version!(0, ProtocolVersionMessage);
message_version!(1, StopMessage);
message_version!(2, RestartMessage);
message_version!(3, StatusMessage);
message_version!(4, SendCommandMessage);
message_version!(5, TimingsMessage);
message_version!(6, LogsMessage, console);
message_version!(7, EndLogsListenerMessage, console);
message_version!(8, ConsoleStatusMessage, console);
message_version!(9, TabCompleteMessage, console);
