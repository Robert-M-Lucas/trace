use std::{error::Error, io, time::Duration};
use std::cmp::{max, min};
use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::time::Instant;
use ::crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use ::crossterm::{event, execute};
use ::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use argh::FromArgs;
use image::{ImageBuffer, Pixel, Rgb};
use itertools::Itertools;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use serde::Deserialize;
use crate::app::App;
use crate::crossterm::run;

mod app;
mod crossterm;
mod ui;
mod custom_map;

/// Demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
}

pub type DATA_TYPE = Rc<Vec<(f32, f32)>>;

fn main() -> Result<(), Box<dyn Error>> {
    // const WIDTH: u32 = 15_000;
    // const HEIGHT: u32 = 7_500;
    /*const RADIUS: u32 = 1_000;

    for i in 0..4 {
        let offset = 90.0 * i as f32;
        let mut image_buffer = ImageBuffer::<Rgb<u8>, _>::new(RADIUS * 2, RADIUS * 2);

        let file = File::open("world_countries.txt")?;
        let reader = BufReader::new(file);

        fn draw(start: Option<(u32, u32)>, end: Option<(u32, u32)>, img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
            if start.is_none() || end.is_none() || true { return; }
            let ((x0, y0), (x1, y1)) = (start.unwrap(), end.unwrap());

            let (mut x0, mut y0) = (x0 as i32, y0 as i32);
            let (mut x1, mut y1) = (x1 as i32, y1 as i32);

            let dx = (x1 - x0).abs();
            let dy = (y1 - y0).abs();
            let sx = if x0 < x1 { 1 } else { -1 };
            let sy = if y0 < y1 { 1 } else { -1 };
            let mut err = dx - dy;

            loop {
                *img.get_pixel_mut(x0 as u32, y0 as u32) = Rgb::from([0, 255, 0]);

                if (x0 == x1) && (y0 == y1) { break; };
                let e2 = 2 * err;
                if e2 > -dy { err -= dy; x0 += sx; }
                if e2 < dx { err += dx; y0 += sy; }
            }
        }

        let mut shape_start = None;
        let mut last_point = None;
        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.trim();

            if line.is_empty() {
                draw(last_point, shape_start, &mut image_buffer);
                shape_start = None;
                last_point = None;
                continue;
            }

            let numbers = line.split(' ').collect_vec();
            if numbers.len() < 2 { continue };

            let Ok(lat) = numbers[0].parse::<f32>() else { continue };
            let Ok(long) = numbers[1].parse::<f32>() else { continue };

            let lat = (((lat + offset) + 180.0) % 360.0) - 180.0;

            if lat.abs() > 90.0 { continue };

            // let x = (((x + 180.0) * ((WIDTH - 1) as f32)) / 360.0) as u32;
            // let y = ((-(y - 90.0) * ((HEIGHT - 1) as f32)) / 180.0) as u32;

            let lat = lat.to_radians();
            let long = long.to_radians();

            let _ = RADIUS as f32 + (RADIUS - 1) as f32 * lat.cos() * long.cos();
            let y = RADIUS as f32 - (RADIUS - 1) as f32 * lat.cos() * long.sin();
            let x = RADIUS as f32 + (RADIUS - 1) as f32 * lat.sin();
            let (x, y) = (x as u32, y as u32);

            draw(last_point, Some((x, y)), &mut image_buffer);

            if shape_start.is_none() {
                shape_start = Some((x, y));
            }
            last_point = Some((x, y));

            *image_buffer.get_pixel_mut(x, y) = Rgb::from([255, 255, 255]);
        }

        draw(shape_start, last_point, &mut image_buffer);

        image_buffer.save(format!("output{i}.png")).unwrap();
    }

    return Ok(());*/

    println!("Preloading...");
    let mut data_countries = Vec::with_capacity(415_000);
    let countries_file = File::open("world_countries.txt")?;

    let mut data_world = Vec::with_capacity(455_000);
    let world_file = File::open("world.txt")?;

    let mut min = 10.1f32;
    let mut max = 0.0f32;

    for (file, data) in [(countries_file, &mut data_countries), (world_file, &mut data_world)] {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let numbers = line.split(' ').collect_vec();
            if numbers.len() < 2 { continue };
            let Ok(lat) = numbers[0].parse::<f32>() else { continue };
            let Ok(long) = numbers[1].parse::<f32>() else { continue };

            let lat = (lat + 180.0) / 360.0;
            let long = -(long - 90.0) / 180.0;


            data.push((lat, long));
        }
    }

    let data_countries = Rc::new(data_countries);
    let data_world = Rc::new(data_world);

    // println!("{:?}", conv_coords(51.500000, -0.125000, 1.0, (0.5, 0.5)));
    // println!("{:?}", conv_coords(160.500000, -0.125000, 1.0, (0.5, 0.5)));
    // return Ok(());

    let cli: Cli = argh::from_env();
    let tick_rate = Duration::from_millis(cli.tick_rate);
    run(tick_rate, true, data_countries, data_world)?;
    Ok(())
}

pub fn conv_coords(long: f32, lat: f32, zoom: f32, pos: (f32, f32)) -> (f32, f32) {
    let long = (long + 180.0) / 360.0;
    let lat = -(lat - 90.0) / 180.0;
    let x = (long - pos.0) * zoom + 0.5;
    let y = (lat - pos.1) * zoom + 0.5;
    (x, 1.0 - y)
}