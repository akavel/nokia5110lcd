//! This example shows powerful PIO module in the RP2040 chip to communicate with WS2812 LED modules.
//! See (https://www.sparkfun.com/categories/tags/ws2812)

#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Output, Level};
use embassy_rp::peripherals::{DMA_CH0, PIO0, PIN_25, USB};
use embassy_rp::pio::{InterruptHandler as PioInt, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_rp::spi::{self, Spi};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInt};
use embassy_time::{Duration, Ticker, Timer, Delay};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

pub mod pcd8544;
use pcd8544::Pcd8544;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInt<PIO0>;
    USBCTRL_IRQ => UsbInt<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

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

    let usb_driver = Driver::new(p.USB, Irqs);
    let _ = spawner.spawn(logger_task(usb_driver));

    let _ = spawner.spawn(rainbow(p.PIN_25, p.DMA_CH0, p.PIO0));

    // let mut lcd_clk   = Output::new(p.PIN_2, Level::Low);
    // let mut lcd_mosi  = Output::new(p.PIN_3, Level::Low);
    let mut lcd_dc    = Output::new(p.PIN_4, Level::Low);
    let mut lcd_ce    = Output::new(p.PIN_5, Level::Low);
    let mut lcd_rst   = Output::new(p.PIN_6, Level::Low);
    let mut lcd_light = Output::new(p.PIN_7, Level::High);

    lcd_light.set_high();

    let mut cfg = spi::Config::default();
    cfg.frequency = 2_000_000;
    let mut spi_bus = Spi::new_blocking_txonly(p.SPI0, p.PIN_22, p.PIN_23, cfg);
    // TODO: should not use new_no_delay but regular new
    let spi_dev = ExclusiveDevice::new_no_delay(spi_bus, lcd_ce);
    // use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
    // let spi_dev = SpiDevice::new(spi_bus, lcd_ce);

    let mut delayer = Delay{};
    let mut lcd = Pcd8544::new(spi_dev, lcd_dc, lcd_rst, &mut delayer).expect("cannot fail");
    // test pattern (50% on)
    let ar: [u8; 42*6*2] = core::array::from_fn(|i| if i%2 == 0 { 0x55u8 } else { 0xAAu8 });
    lcd.data(&ar);

    let mut counter = 0;
    loop {
        counter += 1;
        log::info!("Tick {}", counter);

        // lcd.set_light(counter % 2 == 0);
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
            data[0] = (0,0,0).into(); // temporariy turn off the horrible light...
            ws2812.write(&data).await;

            ticker.next().await;
        }
    }
}

