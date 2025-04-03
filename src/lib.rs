// MicroPython Nokia 5110 PCD8544 84x48 LCD driver
// https://github.com/mcauser/micropython-pcd8544
//
// MIT License
// Copyright (c) 2016-2018 Mike Causer
// Copyright (c) 2025 Mateusz Czapliński
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![no_std]

use embedded_hal_1::{digital::OutputPin, spi::SpiDevice, delay::DelayNs};
pub mod buffer;
pub use buffer::Buffer;

#[derive(Clone, Copy, Debug)]
pub enum Error<SPI, DC, RST> {
    Spi(SPI),
    Dc(DC),
    Rst(RST),
}

pub struct Pcd8544<SPI, DC, RST> {
    spi: SPI,
    dc: DC,
    rst: RST,
    func: u8, // FIXME: what is this stuff?
}

impl<SPI, DC, RST> Pcd8544<SPI, DC, RST>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new(spi: SPI, mut dc: DC, rst: RST, delayer: &mut impl DelayNs) -> Result<Self, Error<SPI::Error, DC::Error, RST::Error>> {
        dc.set_low().map_err(Error::Dc)?;
        let mut dev = Self { spi, dc, rst, func: FUNCTION_SET };
        dev.reset(delayer)?;
        dev.init()?;
        Ok(dev)
    }

    /// issue reset impulse to reset the display
    /// you need to call power_on() or init() to resume
    pub fn reset(&mut self, delayer: &mut impl DelayNs) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.rst.set_high().map_err(Error::Rst)?;
        delayer.delay_us(100);
        self.rst.set_low().map_err(Error::Rst)?;
        delayer.delay_us(100); // reset impulse has to be >100 ns and <100 ms
        self.rst.set_high().map_err(Error::Rst)?;
        delayer.delay_us(100);
        Ok(())
    }

    /// power up, horizontal addressing, basic instruction set
    pub fn init(&mut self) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.func = FUNCTION_SET;
        // FIXME: create a config struct with Default impl and pass it as arg
        self.addressing_horizontal(true);
        self.contrast(0x3f, 0x14, 0x06);
        self.cmd(0x0c); // FIXME: DISPLAY_NORMAL
        self.clear()
    }

    pub fn addressing_horizontal(&mut self, horizontal: bool) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        if horizontal {
            self.func &= !ADDRESSING_VERT;
        } else {
            self.func |= ADDRESSING_VERT;
        }
        self.cmd(self.func)?;
        Ok(())
    }

    fn contrast(&mut self, contrast: u8, bias: u8, temp: u8) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        // extended instruction set is required to set temp, bias and vop
        self.cmd(self.func | EXTENDED_INSTR)?;
        // set temperature coefficient
        self.cmd(temp)?;
        // set bias system (n=3 recommended mux rate 1:40/1:34)
        self.cmd(bias)?;
        // set contrast with operating voltage (0x00~0x7f)
        // 0x00 = 3.00V, 0x3f = 6.84V, 0x7f = 10.68V
        // starting at 3.06V, each bit increments voltage by 0.06V at room temperature
        self.cmd(SET_VOP | contrast)?;
        // revert to basic instruction set
        self.cmd(self.func & !EXTENDED_INSTR)?;
        Ok(())
    }

    // set cursor to column x (0~83), bank y (0~5)
    pub fn position(&mut self, x: u8, y: u8) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.cmd(COL_ADDR | x)?; // set x pos (0~83)
        self.cmd(BANK_ADDR | y)?; // set y pos (0~5)
        Ok(())
    }

    // clear DDRAM, reset x,y position to 0,0
    pub fn clear(&mut self) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        const EMPTY: [u8; BUF_SIZE] = [0; BUF_SIZE];
        self.data(&EMPTY)?;
        self.position(0, 0)?;
        Ok(())
    }

    fn cmd(&mut self, command: u8) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.dc.set_low().map_err(Error::Dc)?;
        self.spi.write(&[command]).map_err(Error::Spi)?;
        Ok(())
    }

    pub fn data(&mut self, data: &[u8]) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.dc.set_high().map_err(Error::Dc)?;
        self.spi.write(data).map_err(Error::Spi)?;
        Ok(())
    }
}

/// Function set 0010 0xxx
// FIXME: what is this stuff?
const FUNCTION_SET: u8     = 0x20;
const POWER_DOWN: u8       = 0x04;
const ADDRESSING_VERT: u8  = 0x02;
const EXTENDED_INSTR: u8   = 0x01;

pub const WIDTH: u8 = 0x54;  // 84
pub const HEIGHT: u8 = 0x30;  // 48
pub const BUF_SIZE: usize = HEIGHT as usize * WIDTH as usize / 8;

// Set operation voltage
const SET_VOP: u8 = 0x80;

// DDRAM addresses
const COL_ADDR: u8  = 0x80; // x pos (0~83)
const BANK_ADDR: u8 = 0x40; // y pos, in banks of 8 rows (0~5)

