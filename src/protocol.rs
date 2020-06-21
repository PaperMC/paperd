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

use crate::messaging::MessageSocket;
use crate::util::{ExitError, ExitValue};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

const PROTOCOL_VERSION: i64 = 1;

pub fn check_jar_protocol<P: AsRef<Path>>(path: P) -> Result<(), ExitValue> {
    let jar_path = path.as_ref();

    let jar_file = fs::File::open(jar_path).conv("Failed to open jar file")?;
    let mut jar_archive = match ZipArchive::new(jar_file) {
        Ok(archive) => archive,
        Err(e) => {
            eprintln!(
                "Failed to open jar file ({}): {}",
                jar_path.to_string_lossy(),
                e
            );
            return Err(ExitValue::Code(1));
        }
    };

    let file_path = "META-INF/io.papermc.paper.daemon.protocol";
    let mut protocol_file = match jar_archive.by_name(file_path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!(
                "The specified jar file ({}) does not have a protocol version file, \
                 it is not compatible with paperd.",
                jar_path.to_string_lossy()
            );
            return Err(ExitValue::Code(1));
        }
    };

    let mut buffer = String::new();
    if let Err(e) = protocol_file.read_to_string(&mut buffer) {
        eprintln!(
            "Failed to read contents of protocol version file in {}: {}",
            jar_path.to_string_lossy(),
            e
        );
        return Err(ExitValue::Code(1));
    }

    return match buffer.trim().parse::<i64>() {
        Ok(protocol) => {
            if protocol != PROTOCOL_VERSION {
                eprintln!(
                    "Protocol versions of paperd and jar file({}) do not match. paperd \
                     protocol version: {}; jar protocol version: {}. Please use a version \
                     of paperd compatible with this build of Paper.",
                    jar_path.to_string_lossy(),
                    PROTOCOL_VERSION,
                    protocol
                );
                Err(ExitValue::Code(1))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            eprintln!(
                "Failed to read protocol version file in jar {}: {}",
                jar_path.to_string_lossy(),
                e
            );
            Err(ExitValue::Code(1))
        }
    };
}

pub fn check_protocol(sock: &MessageSocket) -> Result<(), ExitValue> {
    let message = ProtocolVersionMessage {};
    sock.send_message(&message)?;

    let res = sock.receive_message::<ProtocolVersionMessageResponse>()?;

    if res.protocol_version != PROTOCOL_VERSION {
        eprintln!(
            "The protocol versions of paperd and the specified server do not match. \
             paperd protocol version: {}; server protocol version: {}. Please use a version \
             of paperd compatible with this build of Paper.",
            PROTOCOL_VERSION, res.protocol_version
        );
        return Err(ExitValue::Code(1));
    }

    return Ok(());
}

// Request
#[derive(Serialize)]
pub struct ProtocolVersionMessage {}

// Response
#[derive(Serialize, Deserialize)]
struct ProtocolVersionMessageResponse {
    #[serde(rename = "protocolVersion")]
    protocol_version: i64,
}
