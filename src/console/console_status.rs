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

use crate::messaging::MessageHandler;
use serde::{Deserialize, Serialize};

// Request
#[derive(Serialize)]
pub struct ConsoleStatusMessage {}

impl MessageHandler for ConsoleStatusMessage {
    fn type_id() -> i16 {
        return 8;
    }

    fn expect_response() -> bool {
        return true;
    }
}

// Response
#[derive(Deserialize)]
pub struct ConsoleStatusMessageResponse {
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "players")]
    pub players: i32,
    #[serde(rename = "maxPlayers")]
    pub max_players: i32,
    #[serde(rename = "tps")]
    pub tps: f64,
}
