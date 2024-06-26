use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::{Arc, mpsc};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use itertools::Itertools;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};
use ratatui::widgets::ListState;
use serde::Deserialize;
use crate::DATA_TYPE;

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> Self {
        Self { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct TraceEntry {
    pub no: String,
    pub ip: String,
    pub name: String,
    pub time: String,
    pub lat: f32,
    pub long: f32,
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub enhanced_graphics: bool,
    pub show_countries: bool,
    pub zoom: f32,
    pub map_pos: (f32, f32),
    pub data_countries: DATA_TYPE,
    pub data_world: DATA_TYPE,
    pub input: String,
    pub active_trace: Option<Receiver<TraceEntry>>,
    pub trace_target: Option<String>,
    pub trace_result: Vec<TraceEntry>,
    pub trace_error: Option<Receiver<String>>,
    pub status: String,
    pub error: bool
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool, data_countries: DATA_TYPE, data_world: DATA_TYPE) -> Self {
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Main"]),
            enhanced_graphics,
            show_countries: false,
            zoom: 1.0,
            map_pos: (0.5, 0.5),
            data_countries,
            data_world,
            input: String::new(),
            active_trace: None,
            trace_target: None,
            trace_result: vec![],
            trace_error: None,
            status: "Waiting".to_string(),
            error: false,
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn on_tick(&mut self) {
        if let Some(trace) = &mut self.active_trace {
            if let Ok(data) = trace.try_recv() {
                self.trace_result.push(data);
            }
        }

        if let Some(trace) = &mut self.trace_error {
            if let Ok(error) = trace.try_recv() {
                if &error == "ok" {
                    self.status = "Done".to_string();
                }
                else {
                    self.status = error;
                    self.error = true;
                }
            }
        }
    }

    pub fn trace(&mut self) {
        let (tx, rx) = mpsc::channel();
        let (etx, erx) = mpsc::channel();
        self.status = "In Progress...".to_string();
        self.active_trace = Some(rx);
        self.trace_error = Some(erx);
        self.error = false;
        self.trace_result = Vec::new();
        self.trace_target = Some(self.input.clone());

        let target = self.input.clone();

        self.input = String::new();

        thread::spawn(move || {
            let trace = Command::new("traceroute")
                .args([&target])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn().unwrap();

            let reader = BufReader::new(trace.stdout.unwrap());

            reader.lines()
                .filter_map(|line| line.ok())
                .skip(1)
                .enumerate()
                .for_each(|(i, line)| {
                    let line = line.trim();
                    let split = line.split("  ").skip(1).collect_vec();

                    let mut traces = Vec::new();

                    let mut sections = split.iter();
                    let mut components = sections.next().unwrap().split(" ");
                    loop {
                        let Some(n) = components.next() else {
                            if let Some(ns) = sections.next() {
                                components = ns.split(" ");

                                let mut time = components.next().unwrap().to_string();
                                time += " ";
                                time += components.next().unwrap();
                                traces.push(TraceEntry {
                                    no: format!("{}", i+1),
                                    ip: "-".to_string(),
                                    name: "-".to_string(),
                                    lat: f32::NAN,
                                    long: f32::NAN,
                                    time
                                });
                                continue;
                            }
                            else {
                                break;
                            }
                        };

                        if n == "*" {
                            traces.push(TraceEntry {
                                no: format!("{}", i+1),
                                ip: "x".to_string(),
                                name: "x".to_string(),
                                time: "-".to_string(),
                                lat: f32::NAN,
                                long: f32::NAN,
                            });
                            continue;
                        }

                        let mut name = n.to_string();

                        let ip = components.next().unwrap();
                        let ip = ip[1..ip.len()-1].to_string();

                        if &name == &ip {
                            name = "-".to_string();
                        }

                        components = sections.next().unwrap().split(" ");

                        let mut time = components.next().unwrap().to_string();
                        time += " ";
                        time += components.next().unwrap();

                        #[derive(Debug, Deserialize)]
                        struct Loc {
                            lat: f32,
                            lon: f32
                        }


                        let l = reqwest::blocking::get(format!("http://ip-api.com/json/{ip}?fields=lat,lon"))
                            .and_then(|r| r.json())
                            .unwrap_or( Loc { lat: f32::NAN, lon: f32::NAN } );

                        // let l = Loc { lat: f32::NAN, lon: f32::NAN };

                        traces.push(TraceEntry {
                            no: format!("{}", i+1),
                            ip,
                            name,
                            time,
                            lat: l.lat,
                            long: l.lon,
                        });
                    }

                    if traces.len() > 1 {
                        for (x, trace) in traces.iter_mut().enumerate() {
                            trace.no = format!("{}{}", trace.no, ALPH[x]);
                        }
                    }

                    for trace in traces {
                        tx.send(trace).unwrap();
                    }
                });

            let mut err_string = String::new();
            trace.stderr.unwrap().read_to_string(&mut err_string).unwrap();
            if !err_string.is_empty() {
                etx.send(err_string).unwrap();
            }
            else {
                etx.send("ok".to_string()).unwrap();
            }
        });
    }
}

const ALPH: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];