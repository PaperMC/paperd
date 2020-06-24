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

use crate::console::{
    COLOR_BRIGHT_BLUE, COLOR_BRIGHT_CYAN, COLOR_BRIGHT_GREEN, COLOR_BRIGHT_MAGENTA,
    COLOR_BRIGHT_RED, COLOR_BRIGHT_WHITE, COLOR_BRIGHT_YELLOW, COLOR_DARK_GRAY,
};
use ncurses::{
    attr_t, attroff, attron, chtype, init_pair, mvaddstr, mvhline, A_BLINK, A_BOLD, A_ITALIC,
    A_UNDERLINE, COLOR_BLACK, COLOR_BLUE, COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_PAIR,
    COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};
use std::cmp::min;
use std::slice::Iter;

const BLACK_PAIR: i16 = 1;
const DARK_BLUE_PAIR: i16 = 2;
const DARK_GREEN_PAIR: i16 = 3;
const DARK_AQUA_PAIR: i16 = 4;
const DARK_RED_PAIR: i16 = 5;
const DARK_PURPLE_PAIR: i16 = 6;
const GOLD_PAIR: i16 = 7;
const GRAY_PAIR: i16 = 8;
const DARK_GRAY_PAIR: i16 = 9;
const BLUE_PAIR: i16 = 10;
const GREEN_PAIR: i16 = 11;
const AQUA_PAIR: i16 = 12;
const RED_PAIR: i16 = 13;
const LIGHT_PURPLE_PAIR: i16 = 14;
const YELLOW_PAIR: i16 = 15;
const WHITE_PAIR: i16 = 16;

// These obviously don't cover all ANSI code cases, not even close. These simply represent the
// possible output codes from TerminalConsoleAppender:
//   - https://github.com/Minecrell/TerminalConsoleAppender/blob/b8117c8f0301c832a06c4fcbbf372528a70bcaf4/src/main/java/net/minecrell/terminalconsole/MinecraftFormattingConverter.java#L89-L110
//   - https://github.com/Minecrell/TerminalConsoleAppender/blob/b8117c8f0301c832a06c4fcbbf372528a70bcaf4/src/main/java/net/minecrell/terminalconsole/HighlightErrorConverter.java#L62-L64
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnsiCode {
    Black,         // §0
    DarkBlue,      // §1
    DarkGreen,     // §2
    DarkAqua,      // §3
    DarkRed,       // §4
    DarkPurple,    // §5
    Gold,          // §6
    Gray,          // §7
    DarkGray,      // §8
    Blue,          // §9
    Green,         // §a
    Aqua,          // §b
    Red,           // §c
    LightPurple,   // §d
    Yellow,        // §e
    White,         // §f
    Obfuscated,    // §k
    Bold,          // §l
    Strikethrough, // §m
    Underline,     // §n
    Italic,        // §o
    Reset,         // §r
    Warn,
    Error,
}

impl AnsiCode {
    #[inline]
    fn prefix() -> &'static str {
        return "\u{001B}[";
    }

    #[inline]
    fn suffix() -> &'static str {
        return "m";
    }

    #[inline]
    fn mc_prefix() -> char {
        return '§';
    }

    pub fn init_colors() {
        for code in Self::iter() {
            // Warn and Error use yellow and red colors
            if *code == AnsiCode::Warn || *code == AnsiCode::Error {
                continue;
            }
            if let (Some((pair, fg, bg)), _) = code.attr_pair() {
                init_pair(pair, fg, bg);
            }
        }
    }

    pub fn ansi_code(&self) -> &'static str {
        return match *self {
            AnsiCode::Black => "\u{001B}[0;30m",
            AnsiCode::DarkBlue => "\u{001B}[0;34m",
            AnsiCode::DarkGreen => "\u{001B}[0;32m",
            AnsiCode::DarkAqua => "\u{001B}[0;36m",
            AnsiCode::DarkRed => "\u{001B}[0;31m",
            AnsiCode::DarkPurple => "\u{001B}[0;35m",
            AnsiCode::Gold => "\u{001B}[0;33m",
            AnsiCode::Gray => "\u{001B}[0;37m",
            AnsiCode::DarkGray => "\u{001B}[0;30;1m",
            AnsiCode::Blue => "\u{001B}[0;34;1m",
            AnsiCode::Green => "\u{001B}[0;32;1m",
            AnsiCode::Aqua => "\u{001B}[0;36;1m",
            AnsiCode::Red => "\u{001B}[0;31;1m",
            AnsiCode::LightPurple => "\u{001B}[0;35;1m",
            AnsiCode::Yellow => "\u{001B}[0;33;1m",
            AnsiCode::White => "\u{001B}[0;37;1m",
            AnsiCode::Obfuscated => "\u{001B}[5m",
            AnsiCode::Bold => "\u{001B}[21m",
            AnsiCode::Strikethrough => "\u{001B}[9m",
            AnsiCode::Underline => "\u{001B}[4m",
            AnsiCode::Italic => "\u{001B}[3m",
            AnsiCode::Reset => "\u{001B}[m",
            AnsiCode::Warn => "\u{001B}[31;1m",
            AnsiCode::Error => "\u{001B}[33;1m",
        };
    }

    fn attr_pair(&self) -> (Option<(i16, i16, i16)>, Option<attr_t>) {
        return match *self {
            AnsiCode::Black => (Some((BLACK_PAIR, COLOR_BLACK, COLOR_WHITE)), None),
            AnsiCode::DarkBlue => (Some((DARK_BLUE_PAIR, COLOR_BLUE, -1)), None),
            AnsiCode::DarkGreen => (Some((DARK_GREEN_PAIR, COLOR_GREEN, -1)), None),
            AnsiCode::DarkAqua => (Some((DARK_AQUA_PAIR, COLOR_CYAN, -1)), None),
            AnsiCode::DarkRed => (Some((DARK_RED_PAIR, COLOR_RED, -1)), None),
            AnsiCode::DarkPurple => (Some((DARK_PURPLE_PAIR, COLOR_MAGENTA, -1)), None),
            AnsiCode::Gold => (Some((GOLD_PAIR, COLOR_YELLOW, -1)), None),
            AnsiCode::Gray => (Some((GRAY_PAIR, COLOR_WHITE, COLOR_BLACK)), None),
            AnsiCode::DarkGray => (Some((DARK_GRAY_PAIR, COLOR_DARK_GRAY, COLOR_WHITE)), None),
            AnsiCode::Blue => (Some((BLUE_PAIR, COLOR_BRIGHT_BLUE, -1)), None),
            AnsiCode::Green => (Some((GREEN_PAIR, COLOR_BRIGHT_GREEN, -1)), None),
            AnsiCode::Aqua => (Some((AQUA_PAIR, COLOR_BRIGHT_CYAN, -1)), None),
            AnsiCode::Red => (Some((RED_PAIR, COLOR_BRIGHT_RED, -1)), None),
            AnsiCode::LightPurple => (Some((LIGHT_PURPLE_PAIR, COLOR_BRIGHT_MAGENTA, -1)), None),
            AnsiCode::Yellow => (Some((YELLOW_PAIR, COLOR_BRIGHT_YELLOW, -1)), None),
            AnsiCode::White => (Some((WHITE_PAIR, COLOR_BRIGHT_WHITE, COLOR_BLACK)), None),
            AnsiCode::Obfuscated => (None, Some(A_BLINK())),
            AnsiCode::Bold => (None, Some(A_BOLD())),
            AnsiCode::Strikethrough => (None, None), // ncurses doesn't support strikethrough, so this does nothing
            AnsiCode::Underline => (None, Some(A_UNDERLINE())),
            AnsiCode::Italic => (None, Some(A_ITALIC())),
            AnsiCode::Reset => (None, None), // this is a special case, will cause all other effects to undo
            AnsiCode::Warn => (Some((YELLOW_PAIR, COLOR_YELLOW, -1)), Some(A_BOLD())),
            AnsiCode::Error => (Some((RED_PAIR, COLOR_RED, -1)), Some(A_BOLD())),
        };
    }

    pub fn mc_code(&self) -> &'static str {
        return match *self {
            AnsiCode::Black => "§0",
            AnsiCode::DarkBlue => "§1",
            AnsiCode::DarkGreen => "§2",
            AnsiCode::DarkAqua => "§3",
            AnsiCode::DarkRed => "§4",
            AnsiCode::DarkPurple => "§5",
            AnsiCode::Gold => "§6",
            AnsiCode::Gray => "§7",
            AnsiCode::DarkGray => "§8",
            AnsiCode::Blue => "§9",
            AnsiCode::Green => "§a",
            AnsiCode::Aqua => "§b",
            AnsiCode::Red => "§c",
            AnsiCode::LightPurple => "§d",
            AnsiCode::Yellow => "§e",
            AnsiCode::White => "§f",
            AnsiCode::Obfuscated => "§k",
            AnsiCode::Bold => "§l",
            AnsiCode::Strikethrough => "§m",
            AnsiCode::Underline => "§n",
            AnsiCode::Italic => "§o",
            AnsiCode::Reset => "§r",
            AnsiCode::Warn => "",
            AnsiCode::Error => "",
        };
    }

    fn iter() -> Iter<'static, AnsiCode> {
        static CODES: [AnsiCode; 24] = [
            AnsiCode::Black,
            AnsiCode::DarkBlue,
            AnsiCode::DarkGreen,
            AnsiCode::DarkAqua,
            AnsiCode::DarkRed,
            AnsiCode::DarkPurple,
            AnsiCode::Gold,
            AnsiCode::Gray,
            AnsiCode::DarkGray,
            AnsiCode::Blue,
            AnsiCode::Green,
            AnsiCode::Aqua,
            AnsiCode::Red,
            AnsiCode::LightPurple,
            AnsiCode::Yellow,
            AnsiCode::White,
            AnsiCode::Obfuscated,
            AnsiCode::Bold,
            AnsiCode::Strikethrough,
            AnsiCode::Underline,
            AnsiCode::Italic,
            AnsiCode::Reset,
            AnsiCode::Warn,
            AnsiCode::Error,
        ];
        return CODES.iter();
    }

    fn enable(self) {
        let (pair, attr) = self.attr_pair();
        if let Some((id, _, _)) = pair {
            attron(COLOR_PAIR(id));
        }
        if let Some(attr) = attr {
            attron(attr);
        }
    }

    fn disable(self) {
        let (pair, attr) = self.attr_pair();
        if let Some(attr) = attr {
            attroff(attr);
        }
        if let Some((id, _, _)) = pair {
            attroff(COLOR_PAIR(id));
        }
    }
}

pub enum MessageElement {
    Text(String),
    Code(AnsiCode),
}

pub struct StyledMessage {
    pub messages: Vec<MessageElement>,
}

impl StyledMessage {
    pub fn parse(message: &str) -> Self {
        let mut result = Vec::<MessageElement>::new();

        let mut slice = &message[..];

        'outer: while !slice.is_empty() {
            let found_index = match slice
                .find(AnsiCode::prefix())
                .or_else(|| slice.find(AnsiCode::mc_prefix()))
            {
                Some(i) => i,
                None => {
                    // There aren't any codes to find
                    result.push(MessageElement::Text(slice.to_string()));
                    break;
                }
            };

            if found_index > 0 {
                // Grab text that happened before the found prefix
                result.push(MessageElement::Text(slice[..found_index].to_string()));
            }

            let is_mc_code = slice[found_index..]
                .chars()
                .nth(0)
                .filter(|c| *c == AnsiCode::mc_prefix())
                .is_some();

            // Once we have found the next code, we need to find the end
            let end_index = if is_mc_code {
                // MC codes are only 2 bytes long
                found_index + 2
            } else {
                // We have to actually find the last char for ANSI codes
                match &slice[found_index..].find(AnsiCode::suffix()) {
                    Some(i) => i + found_index,
                    None => {
                        // This doesn't seem to be a valid code...just ignore it then
                        slice = &slice[AnsiCode::prefix().len()..];
                        continue;
                    }
                }
            };

            let code = &slice[found_index..=end_index];
            for ansi_code in AnsiCode::iter() {
                if code == ansi_code.ansi_code() || code == ansi_code.mc_code() {
                    result.push(MessageElement::Code(*ansi_code));
                    slice = &slice[(end_index + 1)..];
                    continue 'outer;
                }
            }

            // If we're gotten here then we didn't find the code
            // in this case, ignore it
            slice = &slice[(end_index + 1)..];
        }

        return StyledMessage { messages: result };
    }

    pub fn output_text(&self, y: i32, x: i32, length: i32) {
        let mut applied_codes = Vec::<AnsiCode>::new();

        let mut index = x;

        for m in &self.messages {
            match m {
                MessageElement::Text(s) => {
                    let s = s.as_str();
                    let to_print = min(length - index, s.len() as i32);
                    if to_print <= 0 {
                        continue;
                    }
                    mvaddstr(y, index, &s[..to_print as usize]);
                    index += to_print;
                }
                MessageElement::Code(c) => {
                    if *c == AnsiCode::Reset {
                        // Undo all still enabled codes
                        applied_codes.into_iter().rev().for_each(AnsiCode::disable);
                        applied_codes = Vec::<AnsiCode>::new();
                    } else {
                        c.enable();
                        applied_codes.push(*c);
                    }
                }
            }
        }

        // Disable any still enabled codes now that we've finished parsing this line
        applied_codes.into_iter().rev().for_each(AnsiCode::disable);

        mvhline(y, index, ' ' as chtype, length - index); // clear rest of row
    }

    pub fn get_string(&self) -> String {
        let mut last_code: Option<AnsiCode> = None;

        let mut result = String::new();

        for m in &self.messages {
            match m {
                MessageElement::Text(s) => {
                    result.push_str(s);
                }
                MessageElement::Code(c) => {
                    result.push_str(c.ansi_code());
                    last_code = Some(*c);
                }
            }
        }

        if let Some(last_code) = last_code {
            if last_code != AnsiCode::Reset {
                result.push_str(AnsiCode::Reset.ansi_code());
            }
        }

        return result;
    }
}
