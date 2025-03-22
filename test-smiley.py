# From: https://github.com/mcauser/micropython-pcd8544

import pcd8544
from machine import Pin, SPI

# spi = SPI(1)
# spi.init(baudrate=2000000, polarity=0, phase=0)
# cs = Pin(2)
# dc = Pin(15)
# rst = Pin(0)


# Connections:
#  5110 | ProMicro RP2040
# ------+----------------
#  RST    pin 6
#  CE     pin 5
#  DC     pin 4
#  Din    CO      (SPI = PICO Peripheral-In/Controller-Out = MOSI Master-Out/Slave-In
#  Clk    SCK     (SCLK = Serial CLocK)
#  BL     pin 7


spi = SPI(0)
spi.init(baudrate=2000000, polarity=0, phase=0)
cs = Pin(5) # a.k.a. CE
dc = Pin(4)
rst = Pin(6)

# backlight on
bl = Pin(7, Pin.OUT, value=1)

lcd = pcd8544.PCD8544(spi, cs, dc, rst)

# test pattern (50% on)
lcd.data(bytearray([0x55, 0xAA] * 42 * 6))

# bitmap smiley (horzontal msb)
lcd.clear()
# draw 8x16 in bank 0 (rows 0..7)
lcd.position(0, 0)
lcd.data(bytearray(b'\xE0\x38\xE4\x22\xA2\xE1\xE1\x61\xE1\x21\xA2\xE2\xE4\x38\xE0\x00'))
# draw 8x16 in bank 1 (rows 8..15)
lcd.position(0, 1)
lcd.data(bytearray(b'\x03\x0C\x10\x21\x21\x41\x48\x48\x48\x49\x25\x21\x10\x0C\x03\x00'))
