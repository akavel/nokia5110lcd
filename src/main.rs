//! This example shows powerful PIO module in the RP2040 chip to communicate with WS2812 LED modules.
//! See (https://www.sparkfun.com/categories/tags/ws2812)

#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Output, Level};
use embassy_rp::peripherals::{DMA_CH0, PIO0, PIN_25};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_time::{Duration, Ticker, Timer};
use smart_leds::RGB8;
use pcd8544::PCD8544;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let _ = spawner.spawn(rainbow(p.PIN_25, p.DMA_CH0, p.PIO0));

    let mut lcd_clk   = Output::new(p.PIN_2, Level::Low);
    let mut lcd_din   = Output::new(p.PIN_3, Level::Low);
    let mut lcd_dc    = Output::new(p.PIN_4, Level::Low);
    let mut lcd_ce    = Output::new(p.PIN_5, Level::Low);
    let mut lcd_rst   = Output::new(p.PIN_6, Level::Low);
    let mut lcd_light = Output::new(p.PIN_7, Level::High);

    let mut lcd = PCD8544::new(
        lcd_clk,
        lcd_din,
        lcd_dc,
        lcd_ce,
        lcd_rst,
        lcd_light,
    ).expect("cannot fail");

    lcd.reset().expect("cannot fail");
    writeln!(lcd, "Hello lcd!");

    loop {
        lcd.set_light(true);
        Timer::after_secs(1).await;
        lcd.set_light(false);
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn rainbow(pin: PIN_25, dma: DMA_CH0, pio: PIO0) {
    // This is the number of leds in the string. Helpfully, the sparkfun thing plus and adafruit
    // feather boards for the 2040 both have one built in.
    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];

    let Pio { mut common, sm0, .. } = Pio::new(pio, Irqs);

    // Common neopixel pins:
    // Thing plus: 8
    // Adafruit Feather: 16;  Adafruit Feather+RFM95: 4
    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, dma, pin, &program);

    // Loop forever making RGB values and pushing them out to the WS2812.
    let mut ticker = Ticker::every(Duration::from_millis(10));
    loop {
        for j in 0..(256 * 5) {
            debug!("New Colors:");
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
                debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
            }
            ws2812.write(&data).await;

            ticker.next().await;
        }
    }
}

