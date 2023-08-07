use anyhow::Result;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::Gpio0;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use st7735_lcd;
use st7735_lcd::Orientation;

use std::f64::consts::PI;

fn generate_cos_wave(frequency: f64, amplitude: f64, sampling_rate: usize, duration: f64) -> Vec<f64> {
    let num_samples = (sampling_rate as f64 * duration) as usize;
    let mut signal = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f64 / sampling_rate as f64;
        signal.push(amplitude * (2.0 * PI * frequency * t).cos());
    }

    signal
}

fn map_values(values: &[f64], min_in: f64, max_in: f64, min_out: i32, max_out: i32) -> Vec<i32> {
    let mut mapped_values = Vec::new();
    for &value in values {
        let normalized_value = (value - min_in) / (max_in - min_in);
        let mapped_value = (normalized_value * (max_out - min_out) as f64 + min_out as f64) as i32;
        mapped_values.push(mapped_value);
    }
    mapped_values
}

fn main() -> Result<()> {

    let f = 10.0; // Frequency in Hz
    let amplitude = 1.0;
    let period = 1.0; // Period in seconds
    let sampling_rate = 160;

    // Generate the signal
    let signal = generate_cos_wave(f, amplitude, sampling_rate, period);

    let min_value = -1.0;
    let max_value = 1.0;
    let min_mapped = 0;
    let max_mapped = 128;

    let signal = map_values(&signal, min_value, max_value, min_mapped, max_mapped);

    print!("{:?}",signal);

    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio8;
    let sdo = peripherals.pins.gpio10;
    let sdi = Option::<Gpio0>::None;
    let cs = Option::<Gpio0>::None;
    let driver_config = Default::default();
    let spi_config = spi::SpiConfig::new().baudrate(30.MHz().into());
    let spi =
        spi::SpiDeviceDriver::new_single(spi, sclk, sdo, sdi, cs, &driver_config, &spi_config)?;

    let rst = PinDriver::output(peripherals.pins.gpio3)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;

    let rgb = true;
    let inverted = false;
    let width = 160;
    let height:i32 = 128;

    let mut delay = FreeRtos;

    let mut display = st7735_lcd::ST7735::new(spi, dc, rst, rgb, inverted, width, height as u32);

    display.init(&mut delay).unwrap();
    let _ = display.set_orientation(&Orientation::Landscape);
    display.clear(Rgb565::BLACK).unwrap();

    let line_style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 7);

    for i in 0..80 {
        Line::new(Point::new(i*2,signal[i as usize]),Point::new((i+1)*2,signal[(i+1) as usize]))
        .into_styled(line_style)
        .draw(&mut display).unwrap();
    }

    println!("lcd test have done.");
    Ok(())

}
