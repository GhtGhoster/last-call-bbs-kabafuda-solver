use std::{thread::sleep, time::Duration};

use enigo::{Coordinate, Enigo, Mouse, Settings};
use screenshots::{image::{io::Reader, GenericImageView, ImageBuffer}, Screen};

const SCREEN_X: i32 = 800;
const SCREEN_Y: i32 = 490;
const SCREEN_W: u32 = 300;
const SCREEN_H: u32 = 60;

fn main() {
    let screens = Screen::all().unwrap();
    let screen = screens[0];

    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let mut image: ImageBuffer<screenshots::image::Rgba<u8>, Vec<u8>> = screen.capture_area(SCREEN_X, SCREEN_Y, SCREEN_W, SCREEN_H).unwrap();

    let i = 0;
    let x = 0;
    let y = 0;
    let comparison_image = Reader::open(format!("assets/{i}.png")).unwrap().decode().unwrap();
    comparison_image.get_pixel(x, y);

    enigo.move_mouse(x as i32, y as i32, Coordinate::Abs).unwrap();
    sleep(Duration::from_millis(50));
    enigo.button(enigo::Button::Left, enigo::Direction::Click).unwrap();
    sleep(Duration::from_millis(50));
}
