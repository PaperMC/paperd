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

pub mod ansi;
mod console_status;

use crate::console::ansi::{AnsiCode, MessageElement, StyledMessage};
use crate::console::console_status::{ConsoleStatusMessage, ConsoleStatusMessageResponse};
use crate::messaging::{open_message_channel, MessageHandler, ResponseChannel};
use crate::protocol::check_protocol;
use crate::send::send_command;
use crate::status::{StatusMessage, StatusMessageResponse};
use crate::util::{get_pid, is_pid_running, ExitError};
use crate::{messaging, util};
use clap::ArgMatches;
use crossbeam_channel::Sender;
use ncurses::{
    addch, addstr, attroff, attron, chtype, delscreen, delwin, echochar, endwin, getcurx, getmaxyx,
    halfdelay, has_colors, hline, init_pair, initscr, keypad, mvaddch, mvaddstr, mvdelch, mvgetch,
    mvhline, mvwaddstr, mvwhline, mvwvline, newwin, noecho, refresh, start_color, stdscr, touchwin,
    use_default_colors, wattroff, wattron, werase, wrefresh, COLOR_BLACK, COLOR_BLUE, COLOR_GREEN,
    COLOR_MAGENTA, COLOR_PAIR, COLOR_RED, COLOR_YELLOW, ERR, KEY_BACKSPACE, KEY_DOWN, KEY_ENTER,
    KEY_EVENT, KEY_F1, KEY_F2, KEY_LEFT, KEY_RESIZE, KEY_RIGHT, KEY_UP, WINDOW,
};
use nix::sys::signal::kill;
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use signal_hook::iterator::Signals;
use signal_hook::{SIGABRT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTRAP};
use std::cmp::min;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::sleep;
use std::time::Duration;
use std::vec::Vec;
use std::{fs, str, thread};

macro_rules! ctrl {
    ($letter:expr) => {
        ((($letter as u8) as i32) & 0x1F)
    };
}

const CTRL_B: i32 = ctrl!('b');
const CTRL_Q: i32 = ctrl!('q');
const CTRL_F: i32 = ctrl!('f');
const KEY_TAB: i32 = '\t' as u8 as i32;

const NORMAL_KEY_ENTER: i32 = 10;
const NORMAL_KEY_BACKSPACE: i32 = 127;

const PROMPT_PAIR: i16 = 25;
const STATUS_PAIR: i16 = 26;
const SELECTED_PAIR: i16 = 27;
const DECENT_TPS: i16 = 28;
const BAD_TPS: i16 = 29;

const COMPLETE_BORDER_PAIR: i16 = 30;
const COMPLETE_TEXT_PAIR: i16 = 31;
const COMPLETE_SELECTED_PAIR: i16 = 32;

const COLOR_DARK_GRAY: i16 = 8;
const COLOR_BRIGHT_RED: i16 = 9;
const COLOR_BRIGHT_GREEN: i16 = 10;
const COLOR_BRIGHT_YELLOW: i16 = 11;
const COLOR_BRIGHT_BLUE: i16 = 12;
const COLOR_BRIGHT_MAGENTA: i16 = 13;
const COLOR_BRIGHT_CYAN: i16 = 14;
const COLOR_BRIGHT_WHITE: i16 = 15;

macro_rules! prompt_line {
    ($max_y:ident) => {
        $max_y - 2
    };
}

macro_rules! prompt_index {
    ($index:ident) => {
        ($index + 2) as i32
    };
}

#[cfg(feature = "console")]
pub fn console(sub_m: &ArgMatches) -> Result<(), i32> {
    let (pid_file, pid) = get_pid(sub_m)?;
    check_protocol(&pid_file)?;

    // Open logs channel
    let chan = messaging::open_message_channel(&pid_file)?;
    {
        let message = StatusMessage {};
        let response_chan = chan
            .send_message::<StatusMessage>(message)?
            .expect("Failed to create response channel");
        if let Err(_) = response_chan.receive_message::<StatusMessageResponse>() {
            // Server is not yet ready (which will already be told to the user by the above
            // function) so just quit before creating the console
            return Err(1);
        }
    }

    let message = LogsMessage {};
    let response_chan = chan
        .send_message::<LogsMessage>(message)?
        .expect("Failed to create response channel");

    let res = Term::new(&pid_file, &response_chan).run_term();

    if is_pid_running(pid) {
        let end = EndLogsListenerMessage {
            channel: response_chan.response_chan,
        };
        messaging::open_message_channel(&pid_file)?.send_message::<EndLogsListenerMessage>(end)?;
    }

    return res;
}

struct Term<'a> {
    pid_file: &'a PathBuf,
    resp_chan: &'a ResponseChannel,
    signals: Signals,
    completions: Option<Completions>,
}

impl<'a> Term<'a> {
    fn new(pid_file: &'a PathBuf, resp_chan: &'a ResponseChannel) -> Self {
        return Term {
            pid_file,
            resp_chan,
            signals: Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGTRAP, SIGABRT, SIGTERM]).unwrap(),
            completions: None,
        };
    }

    fn run_term(self) -> Result<(), i32> {
        // Start ncurses
        initscr();
        if !has_colors() {
            eprintln!("Your terminal is not supported");
            return Err(1);
        }

        keypad(stdscr(), true);
        noecho();

        start_color();
        use_default_colors();

        init_pair(PROMPT_PAIR, COLOR_GREEN, -1);
        init_pair(STATUS_PAIR, COLOR_BLACK, COLOR_BLUE);
        init_pair(SELECTED_PAIR, COLOR_BLACK, COLOR_GREEN);
        init_pair(DECENT_TPS, COLOR_BLACK, COLOR_YELLOW);
        init_pair(BAD_TPS, COLOR_BLACK, COLOR_RED);
        init_pair(COMPLETE_BORDER_PAIR, COLOR_MAGENTA, -1);
        init_pair(COMPLETE_TEXT_PAIR, COLOR_YELLOW, -1);
        init_pair(COMPLETE_SELECTED_PAIR, COLOR_YELLOW, COLOR_DARK_GRAY);

        ansi::AnsiCode::init_colors();

        return self.do_term_loop();
    }

    fn do_term_loop(self) -> Result<(), i32> {
        let stop = Arc::new(AtomicBool::new(false));

        // line buffer, holds the log messages we receive from the server
        let buffer = Arc::new(Mutex::new(Vec::<StyledMessage>::new()));

        // Set up listeners
        self.start_stop_listener_thread(stop.clone())?;
        self.start_signals_listener_thread(stop.clone());

        self.start_new_message_listener_thread(stop.clone(), buffer.clone());

        let status = Arc::new(Mutex::new(CurrentStatus {
            mode: ArrowMode::INPUT,
            tps: 0.0,
            players: 0,
            max_players: 0,
            server_name: "".to_string(),
        }));

        self.start_status_bar_thread(stop.clone(), status.clone());

        return self.input_loop(stop.clone(), buffer.clone(), status.clone());
    }

    fn start_stop_listener_thread(&self, stop: Arc<AtomicBool>) -> Result<(), i32> {
        let pid_text = fs::read_to_string(&self.pid_file).conv("Failed to read PID file")?;
        let pid_int = pid_text.parse::<i32>().conv("Failed to parse PID file")?;
        let pid = Pid::from_raw(pid_int);
        thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                if let Err(_) = kill(pid, None) {
                    stop.store(true, Ordering::SeqCst);
                    break;
                } else {
                    sleep(Duration::from_secs(2));
                }
            }
        });

        return Ok(());
    }

    fn start_signals_listener_thread(&self, stop: Arc<AtomicBool>) {
        let signals_bg = self.signals.clone();
        thread::spawn(move || {
            for _ in signals_bg.forever() {
                stop.store(true, Ordering::SeqCst);
                break;
            }
        });
    }

    fn start_new_message_listener_thread(
        &self,
        stop: Arc<AtomicBool>,
        buffer: Arc<Mutex<Vec<StyledMessage>>>,
    ) {
        let chan_bg = self.resp_chan.clone();

        thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                let res = match chan_bg.receive_message::<LogsMessageResponse>() {
                    Ok(res) => res,
                    Err(_) => {
                        stop.store(true, Ordering::SeqCst);
                        break;
                    }
                };

                let mut code_hist = Vec::<AnsiCode>::new();

                // Multi-line messages can have styles at the start and RESET at the end, which would
                // be expected to be applied to the whole block
                // But we split these messages up into their own individual lines to make displaying
                // them easier, so need to essentially "re-apply" these styles on every line
                //
                // The key is to make sure we still respect RESET tokens when they appear
                for part in res.message.split_terminator('\n') {
                    let mut next_code_hist = Vec::<AnsiCode>::new();

                    let mut msg = StyledMessage::parse(part.replace("\t", "    ").as_str());
                    // Figure out which parts leak into other lines
                    for element in &msg.messages {
                        if let MessageElement::Code(c) = element {
                            if *c == AnsiCode::Reset {
                                next_code_hist.clear();
                                code_hist.clear();
                            } else {
                                next_code_hist.push(*c);
                            }
                        }
                    }

                    for code in &code_hist {
                        msg.messages.insert(0, MessageElement::Code(*code));
                    }
                    msg.messages.push(MessageElement::Code(AnsiCode::Reset));
                    buffer.lock().unwrap().push(msg);

                    code_hist.append(&mut next_code_hist);
                }
            }
        });
    }

    fn start_status_bar_thread(&self, stop: Arc<AtomicBool>, status: Arc<Mutex<CurrentStatus>>) {
        let pid_file_bg = self.pid_file.clone();
        thread::spawn(move || {
            macro_rules! handle_error {
                ($stop:ident) => {
                    if $stop.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                    continue;
                };
            }

            while !stop.load(Ordering::SeqCst) {
                let chan = match open_message_channel(&pid_file_bg) {
                    Ok(c) => c,
                    Err(_) => {
                        handle_error!(stop);
                    }
                };

                let resp: ConsoleStatusMessageResponse = {
                    let response_chan = match chan.send_message(ConsoleStatusMessage {}) {
                        Ok(s) => s.expect("Failed to create response channel"),
                        Err(_) => {
                            handle_error!(stop);
                        }
                    };

                    match response_chan.receive_message::<ConsoleStatusMessageResponse>() {
                        Ok(r) => r,
                        Err(_) => {
                            handle_error!(stop);
                        }
                    }
                };

                {
                    let mut status = status.lock().unwrap();
                    status.server_name = resp.server_name;
                    status.players = resp.players;
                    status.max_players = resp.max_players;
                    status.tps = resp.tps;
                }

                if stop.load(Ordering::SeqCst) {
                    break;
                }

                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    fn input_loop(
        mut self,
        stop: Arc<AtomicBool>,
        buffer: Arc<Mutex<Vec<StyledMessage>>>,
        status: Arc<Mutex<CurrentStatus>>,
    ) -> Result<(), i32> {
        // The server response results of the completion requests
        let (comp_res_send, comp_res_rec) = crossbeam_channel::unbounded::<Vec<String>>();

        // index represents the last line visible on screen
        // it's subtracted from the buffer's length to find the line
        // buffer.len() - 1 - index
        let mut index = 0;
        // cursor_index represents where on the input line the cursor is
        // it's 1:1 with the input variable, which is offset by 2 from the left due to the '> ' prompt
        // So the actual cursor index is 2 + cursor_index
        let mut cursor_index = 0;

        let mut input_history_up = Vec::<String>::new();
        let mut input_history_down = Vec::<String>::new();

        let mut input = Vec::<char>::new();

        // wait 10 ms for inputs
        halfdelay(1);

        let mut last_len = std::usize::MAX;
        let mut last_index = index;

        let mut last_max_x = -1;
        let mut last_max_y = -1;

        while !stop.load(Ordering::SeqCst) {
            // Get screen bounds
            let mut max_x = 0;
            let mut max_y = 0;
            getmaxyx(stdscr(), &mut max_y, &mut max_x);

            {
                let buf = buffer.lock().unwrap();
                let len = buf.len();
                if last_len != len
                    || last_index != index
                    || last_max_x != max_x
                    || last_max_y != max_y
                {
                    if index != 0 && last_index == index {
                        // if index is not 0 (that is, we're not following the log at the bottom)
                        // then we want to keep track of where it was and adjust accordingly
                        let len_diff = len - last_len;
                        index += len_diff;
                    }
                    last_len = len;
                    last_index = index;
                    last_max_x = max_x;
                    last_max_y = max_y;
                    redraw_term(&buf, &input, max_x, max_y, index);
                }
            }

            if let Some(comp) = &self.completions {
                comp.redraw();
            }

            // Wait for input
            while !stop.load(Ordering::SeqCst) {
                // there are lines to draw
                if buffer.lock().unwrap().len() != last_len {
                    break;
                }

                let mut cur_max_x = 0;
                let mut cur_max_y = 0;
                getmaxyx(stdscr(), &mut cur_max_y, &mut cur_max_x);
                // terminal size has changed, we need to redraw
                if cur_max_x != max_x || cur_max_y != max_y {
                    break; // redraw
                }

                status.lock().unwrap().status_line(max_y, max_x);

                if let Ok(suggestions) = comp_res_rec.try_recv() {
                    self.completions = Completions::new(max_y, max_x, suggestions);
                    if let Some(comp) = &self.completions {
                        comp.redraw();
                    }
                }

                let ch = mvgetch(prompt_line!(max_y), prompt_index!(cursor_index));
                match ch {
                    KEY_RESIZE => {
                        break; // redraw
                    }
                    ERR | KEY_EVENT => {
                        continue;
                    }
                    _ => {}
                }

                if let Some(comp) = &mut self.completions {
                    let (command, action) = comp.handle_key(ch);

                    if action & Completions::CLOSE_WINDOW != 0 {
                        self.completions = None;
                    }

                    if let Some(text) = command {
                        let input_text: String = input.into_iter().collect();
                        let split: Vec<&str> = input_text.split(" ").collect();

                        input = Vec::<char>::new();
                        if split.is_empty() {
                            input = text.chars().collect();
                        } else {
                            for (i, part) in split.iter().enumerate() {
                                let new_part = if i == split.len() - 1 {
                                    text.as_str()
                                } else {
                                    part
                                };
                                if i != 0 {
                                    input.push(' ');
                                }
                                for c in new_part.chars() {
                                    input.push(c);
                                }
                            }
                        }
                        cursor_index = input.len();
                        prompt(&input, max_y, max_x);
                        refresh();
                    }

                    if action & Completions::SEND_KEY == 0 {
                        continue;
                    }
                }

                match ch {
                    KEY_F1 => {
                        status.lock().unwrap().mode = ArrowMode::INPUT;
                    }
                    KEY_F2 => {
                        status.lock().unwrap().mode = ArrowMode::SCROLL;
                    }
                    KEY_UP => {
                        match status.lock().unwrap().mode {
                            ArrowMode::SCROLL => {
                                let len = buffer.lock().unwrap().len() as i32;
                                if len - index as i32 > max_y - 1 {
                                    index += 1;
                                    break; // redraw
                                }
                            }
                            ArrowMode::INPUT => {
                                if input_history_up.is_empty() {
                                    continue;
                                }

                                let input_text: String = input.into_iter().collect();
                                if !input_text.is_empty() {
                                    input_history_down.push(input_text);
                                }
                                input = input_history_up.pop().unwrap().chars().collect();
                                cursor_index = input.len();
                                prompt(&input, max_y, max_x);
                                refresh();
                            }
                        }
                    }
                    KEY_DOWN => {
                        match status.lock().unwrap().mode {
                            ArrowMode::SCROLL => {
                                if index > 0 {
                                    index -= 1;
                                    break; // redraw
                                }
                            }
                            ArrowMode::INPUT => {
                                let input_text: String = input.into_iter().collect();
                                if !input_text.is_empty() {
                                    input_history_up.push(input_text);
                                }
                                if input_history_down.is_empty() {
                                    input = Vec::<char>::new();
                                } else {
                                    input = input_history_down.pop().unwrap().chars().collect();
                                }
                                cursor_index = input.len();
                                prompt(&input, max_y, max_x);
                                refresh();
                            }
                        }
                    }
                    KEY_LEFT => {
                        if cursor_index > 0 {
                            cursor_index -= 1;
                            continue;
                        }
                    }
                    KEY_RIGHT => {
                        if cursor_index < input.len() {
                            cursor_index += 1;
                            continue;
                        }
                    }
                    NORMAL_KEY_BACKSPACE | KEY_BACKSPACE => {
                        if cursor_index == 0 {
                            continue;
                        }
                        if cursor_index >= input.len() {
                            input.pop();
                            cursor_index = input.len();
                            mvdelch(prompt_line!(max_y), prompt_index!(cursor_index));
                        } else {
                            input.remove(cursor_index - 1);
                            cursor_index -= 1;
                            prompt(&input, max_y, max_x);
                        }
                        if input.len() == 0 {
                            self.completions = None
                        } else {
                            request_completions(&input, &self.pid_file, &comp_res_send);
                        }

                        refresh();
                    }
                    NORMAL_KEY_ENTER | KEY_ENTER => {
                        // line feed
                        let s: String = input.into_iter().collect();
                        input = Vec::<char>::new();
                        cursor_index = 0;
                        prompt(&input, max_y, max_x);
                        refresh();

                        // Send command last so the prompt isn't waiting to redraw
                        // drain down history into up
                        if !s.is_empty() {
                            send_command(&self.pid_file, s.as_str())?;
                            while !input_history_down.is_empty() {
                                input_history_up.push(input_history_down.pop().unwrap());
                            }
                            input_history_up.push(s);
                        }
                    }
                    CTRL_B | CTRL_Q => {
                        stop.store(true, Ordering::SeqCst);
                        break;
                    }
                    CTRL_F => {
                        // follow
                        index = 0;
                        break; // redraw
                    }
                    ch => {
                        let rs_ch = match std::char::from_u32(ch as u32) {
                            Some(c) => c,
                            None => continue,
                        };
                        if rs_ch.is_alphanumeric()
                            || rs_ch.is_whitespace()
                            || rs_ch.is_ascii_punctuation()
                        {
                            if cursor_index >= input.len() {
                                // efficient case, character goes at end of input line
                                input.push(rs_ch);
                                echochar(ch as chtype);
                                // this also functions as a catch-all for if the cursor_index has
                                // somehow gotten screwed up
                                cursor_index = input.len();
                            } else {
                                input.insert(cursor_index, rs_ch);
                                cursor_index += 1;
                                // this case is less efficient, we'll need to re-render the prompt
                                prompt(&input, max_y, max_x);
                                refresh();
                            }
                        }

                        request_completions(&input, &self.pid_file, &comp_res_send);
                    }
                }
            }
        }

        return Ok(());
    }
}

impl<'a> Drop for Term<'a> {
    fn drop(&mut self) {
        self.completions = None; // force drop now
        self.signals.close();
        endwin();
        delscreen(stdscr());
    }
}

struct Completions {
    window: WINDOW,
    suggestions: Vec<String>,
    index: Option<usize>,
    width: i32,
    height: i32,
    lines: usize,
}

impl Completions {
    fn new(max_y: i32, max_x: i32, suggestions: Vec<String>) -> Option<Completions> {
        // If the current window is too small, we won't show the tab complete popup
        if max_y < 15 || max_x < 40 {
            return None;
        }

        if suggestions.len() == 0 {
            return None;
        }

        let lines = min(suggestions.len() as i32, min(max_y - 5, 15)) as usize;

        let width: i32 = 35;
        let height: i32 = (lines + 2) as i32;

        let new_win = newwin(height, width, prompt_line!(max_y) - height, 2);
        return Some(Completions {
            window: new_win,
            suggestions,
            index: None,
            width,
            height,
            lines,
        });
    }

    fn redraw(&self) {
        werase(self.window);
        wattron(self.window, COLOR_PAIR(COMPLETE_BORDER_PAIR));
        mvwhline(self.window, 0, 0, '*' as chtype, self.width);
        mvwhline(self.window, self.height - 1, 0, '*' as chtype, self.width);
        mvwvline(self.window, 0, 0, '*' as chtype, self.height);
        mvwvline(self.window, 0, self.width - 1, '*' as chtype, self.height);
        wattroff(self.window, COLOR_PAIR(COMPLETE_BORDER_PAIR));

        wattron(self.window, COLOR_PAIR(COMPLETE_TEXT_PAIR));
        for (i, suggestion) in self.suggestions.iter().enumerate() {
            if i >= self.lines {
                break;
            }
            let current_index_selected = self.index.is_some() && i == self.index.unwrap();
            if current_index_selected {
                wattroff(self.window, COLOR_PAIR(COMPLETE_TEXT_PAIR));
                wattron(self.window, COLOR_PAIR(COMPLETE_SELECTED_PAIR));
            }
            mvwaddstr(
                self.window,
                self.height - 2 - i as i32, // - 2 because the first row is for the border
                1,
                suggestion.as_str(),
            );
            if current_index_selected {
                wattroff(self.window, COLOR_PAIR(COMPLETE_SELECTED_PAIR));
                wattron(self.window, COLOR_PAIR(COMPLETE_TEXT_PAIR));
            }
        }
        wattroff(self.window, COLOR_PAIR(COMPLETE_TEXT_PAIR));

        wrefresh(self.window);
    }

    fn handle_key(&mut self, ch: i32) -> (Option<String>, u8) {
        match ch {
            KEY_UP => {
                if self.index.is_none() || self.index.unwrap() < self.lines - 1 {
                    self.index = Some(self.index.unwrap_or(0) + 1);
                    self.redraw();
                }
            }
            KEY_DOWN => {
                if self.index.is_some() && self.index.unwrap() > 0 {
                    self.index = Some(self.index.unwrap() - 1);
                    self.redraw();
                }
            }
            KEY_TAB => {
                let result = self.suggestions[self.index.unwrap_or(0)].clone();
                return (Some(result), Completions::NO_ACTION);
            }
            NORMAL_KEY_ENTER | KEY_ENTER => {
                if self.index.is_none() {
                    return (None, Completions::CLOSE_WINDOW | Completions::SEND_KEY);
                } else {
                    let result = self.suggestions[self.index.unwrap()].clone();
                    return (Some(result), Completions::NO_ACTION);
                }
            }
            27 | CTRL_F => {
                // 27 represents ESC
                return (None, Completions::CLOSE_WINDOW);
            }
            CTRL_B | CTRL_Q => {
                return (None, Completions::CLOSE_WINDOW | Completions::SEND_KEY);
            }
            NORMAL_KEY_BACKSPACE | KEY_BACKSPACE => {
                return (None, Completions::SEND_KEY);
            }
            _ => {
                let rs_ch = match std::char::from_u32(ch as u32) {
                    Some(c) => c,
                    None => return (None, Completions::CLOSE_WINDOW),
                };
                if rs_ch.is_alphanumeric() || rs_ch.is_whitespace() || rs_ch.is_ascii_punctuation()
                {
                    return (None, Completions::SEND_KEY);
                }
            }
        }

        return (None, Completions::NO_ACTION);
    }

    const NO_ACTION: u8 = 0x00;
    const CLOSE_WINDOW: u8 = 0x01;
    const SEND_KEY: u8 = 0x02;
}

impl Drop for Completions {
    fn drop(&mut self) {
        delwin(self.window);
        touchwin(stdscr());
        refresh();
    }
}

fn request_completions(input: &Vec<char>, pid_file: &PathBuf, chan: &Sender<Vec<String>>) {
    let command_text: String = input.iter().collect();
    if command_text.len() == 0 {
        return;
    }

    let pid_file_bg = pid_file.clone();
    let chan_bg = chan.clone();
    thread::spawn(move || {
        let channel = match open_message_channel(&pid_file_bg) {
            Ok(channel) => channel,
            Err(_) => return,
        };

        let message = TabCompleteMessage {
            command: command_text,
        };
        let response = match channel.send_message::<TabCompleteMessage>(message) {
            Ok(resp) => resp.expect("Failed to create response channel"),
            Err(_) => return,
        };

        let received = match response.receive_message::<TabCompleteMessageResponse>() {
            Ok(resp) => resp,
            Err(_) => return,
        };

        let _ = chan_bg.try_send(received.suggestions);
    });
}

fn redraw_term(
    buf: &MutexGuard<Vec<StyledMessage>>,
    cur_input: &Vec<char>,
    max_x: i32,
    max_y: i32,
    index: usize,
) {
    // First, print prompt at bottom of screen
    prompt(cur_input, max_y, max_x);

    let lines = min(max_y - 1, (buf.len() - index) as i32);
    for i in 0..lines {
        // - 3 because first line is always status, second is always prompt
        let cur_y = max_y - i - 3;
        // buffer.len() - 1 gets the last line
        // - index gets the last line of the section we're looking at
        // - i moves up to the line we're printing now
        buf[buf.len() - 1 - index - (i as usize)].output_text(cur_y, 0, max_x);
    }

    refresh();
}

fn prompt(cur_input: &Vec<char>, max_y: i32, max_x: i32) {
    attron(COLOR_PAIR(PROMPT_PAIR));
    mvaddstr(prompt_line!(max_y), 0, "> ");
    attroff(COLOR_PAIR(PROMPT_PAIR));

    if !cur_input.is_empty() {
        let s: String = cur_input.iter().collect();
        // 2 because we're adding after the '> ' prompt
        mvaddstr(prompt_line!(max_y), 2, s.as_str());
        let index = (2 + s.len()) as i32;
        mvhline(prompt_line!(max_y), index, ' ' as chtype, max_x - index); // clear rest of row
    } else {
        mvhline(prompt_line!(max_y), 2, ' ' as chtype, max_x - 2); // clear whole row
    }
}

#[derive(Copy, Clone, PartialEq)]
enum ArrowMode {
    SCROLL,
    INPUT,
}

struct CurrentStatus {
    mode: ArrowMode,
    tps: f64,
    players: i32,
    max_players: i32,
    server_name: String,
}

impl CurrentStatus {
    fn status_line(&self, max_y: i32, max_x: i32) {
        attron(COLOR_PAIR(STATUS_PAIR));

        macro_rules! selected {
            ($name:expr, $text:expr) => {
                if self.mode == $name {
                    attroff(COLOR_PAIR(STATUS_PAIR));
                    attron(COLOR_PAIR(SELECTED_PAIR));
                    addch('*' as chtype);
                } else {
                    addch(' ' as chtype);
                }

                addstr(concat!(" ", $text));
                if self.mode == $name {
                    attroff(COLOR_PAIR(SELECTED_PAIR));
                    attron(COLOR_PAIR(STATUS_PAIR));
                }
            };
        }

        mvaddch(max_y - 1, 0, ' ' as chtype);
        selected!(ArrowMode::INPUT, "Input Mode (F1)");

        addch(' ' as chtype);
        selected!(ArrowMode::SCROLL, "Scroll Mode (F2)");

        let name_len = self.server_name.len() as i32;

        let mut tps = String::from("TPS ");
        let tps_text = format!("{:.2}", util::tps_cap(self.tps));
        tps.push_str(tps_text.as_str());
        let tps_len = tps.len() as i32;

        let mut players_text = self.players.to_string();
        players_text.push_str(" / ");
        players_text.push_str(self.max_players.to_string().as_str());
        let players_len = players_text.len() as i32;

        // - 7 as there's 2 3-char wide gaps (before and after player count) and 1 more at the end
        let index = max_x - 7 - tps_len - name_len - players_len;
        let cur_index = getcurx(stdscr());
        hline(' ' as chtype, index - cur_index);

        mvaddstr(max_y - 1, index, self.server_name.as_str());
        addstr("   ");
        addstr(players_text.as_str());
        addstr("   ");

        macro_rules! swap {
            ($func:ident) => {
                if self.tps < 15.0 {
                    $func(COLOR_PAIR(BAD_TPS));
                } else if self.tps < 19.0 {
                    $func(COLOR_PAIR(DECENT_TPS));
                } else {
                    $func(COLOR_PAIR(SELECTED_PAIR));
                }
            };
        }

        attroff(COLOR_PAIR(STATUS_PAIR));
        swap!(attron);

        addstr(tps.as_str());

        swap!(attroff);
        attron(COLOR_PAIR(STATUS_PAIR));

        addch(' ' as chtype);

        attroff(COLOR_PAIR(STATUS_PAIR));
    }
}

// LogsMessage
#[derive(Serialize)]
struct LogsMessage {}

impl MessageHandler for LogsMessage {
    fn type_id() -> i16 {
        return 6;
    }

    fn expect_response() -> bool {
        return true;
    }
}

#[derive(Deserialize)]
struct LogsMessageResponse {
    #[serde(rename = "message")]
    message: String,
}

// EndLogsListenerMessage
#[derive(Serialize)]
struct EndLogsListenerMessage {
    #[serde(rename = "channel")]
    channel: i32,
}

impl MessageHandler for EndLogsListenerMessage {
    fn type_id() -> i16 {
        return 7;
    }

    fn expect_response() -> bool {
        return false;
    }
}

// AutocompleteMessage
#[derive(Serialize)]
struct TabCompleteMessage {
    #[serde(rename = "command")]
    command: String,
}

impl MessageHandler for TabCompleteMessage {
    fn type_id() -> i16 {
        return 9;
    }

    fn expect_response() -> bool {
        return true;
    }
}

#[derive(Deserialize)]
struct TabCompleteMessageResponse {
    #[serde(rename = "suggestions")]
    suggestions: Vec<String>,
}
