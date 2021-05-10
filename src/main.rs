#![no_std]
#![no_main]

// Pull in the panic handler from panic-halt
// extern crate panic_halt;
extern crate avr_std_stub;
extern crate ufmt;

use arduino_leonardo::prelude::*;
use arduino_leonardo::I2cMaster;


fn write_i2c_extender<M>(i2c: &mut I2cMaster<M>, data: u8) {
    let addr7 = 0x27;
    i2c.write(addr7, &[data]);
}

fn write_nibble<M>(i2c: &mut I2cMaster<M>, rs: bool, rw: bool, backlight: bool, nibble: u8, delay_us: u16) {
    let data = (nibble & 0x0F) << 4;

    let mut ctrl = 0x00;
    if backlight {
        ctrl |= 0x08;
    }
    if rw {
        ctrl |= 0x02;
    }
    if rs {
        ctrl |= 0x01;
    }

    let enable = 0x04;

    write_i2c_extender(i2c, ctrl | data);
    arduino_leonardo::delay_us(1);
    write_i2c_extender(i2c, ctrl | data | enable);
    arduino_leonardo::delay_us(50);
    write_i2c_extender(i2c, ctrl | data);

    arduino_leonardo::delay_us(delay_us);
}

fn write_byte<M>(i2c: &mut I2cMaster<M>, rs: bool, rw: bool, backlight: bool, data: u8) {
    write_nibble(i2c, rs, rw, backlight, (data & 0xf0) >> 4, 0);
    write_nibble(i2c, rs, rw, backlight, (data & 0x0f), 0);
}


fn lcd_init<M>(mut i2c: &mut I2cMaster<M>) {
// go into 4bit mode, 2line, 5x8
    write_nibble(&mut i2c, false, false, true, 0x3, 4500);
    write_nibble(&mut i2c, false, false, true, 0x3, 4500);
    write_nibble(&mut i2c, false, false, true, 0x3, 150);
    write_nibble(&mut i2c, false, false, true, 0x2, 100);

    // set function
    write_byte(&mut i2c, false, false, false, 0x28);
    arduino_leonardo::delay_us(500);
}

fn lcd_displaycontrol<M>(mut i2c: &mut I2cMaster<M>, display_on: bool, cursor: bool, blink: bool) {
    // set mode ltr & screen fixed
    write_byte(&mut i2c, false, false, true, 0x06);

    // set cursor blink
    let mut displaycontrol = 0x8;
    if display_on {
        displaycontrol |= 0x4;
    }
    if cursor {
        displaycontrol |= 0x2;
    }
    if blink {
        displaycontrol |= 0x1;
    }

    write_byte(&mut i2c, false, false, true, displaycontrol);
}

fn lcd_clear<M>(mut i2c: &mut I2cMaster<M>) {

    // clear screen
    write_byte(&mut i2c, false, false, true, 0x01);
    arduino_leonardo::delay_us(500);

    // reset cursor
    write_byte(&mut i2c, false, false, true, 0x02);
}

fn lcd_write_string<M>(mut i2c: &mut I2cMaster<M>, s: &str) {
    for s in s.bytes() {
        write_byte(i2c, true, false, true, s);
    }
}

fn lcd_set_position<M>(mut i2c: &mut I2cMaster<M>, row: u8, col: u8){
    let addr = (row * 0x40 + col) & 0x7f;
    write_byte(i2c, false, false, true, 0x80 | addr);
}


#[arduino_leonardo::entry]
fn main() -> ! {
    let dp = arduino_leonardo::Peripherals::take().unwrap();
    let mut pins = arduino_leonardo::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD, dp.PORTE, dp.PORTF);

    let mut serial = arduino_leonardo::Serial::new(
        dp.USART1,
        pins.d0,
        pins.d1.into_output(&mut pins.ddr),
        57600.into_baudrate(),
    );

    let mut i2c = arduino_leonardo::I2cMaster::new(
        dp.TWI,
        pins.d2.into_pull_up_input(&mut pins.ddr),
        pins.d3.into_pull_up_input(&mut pins.ddr),
        50000,
    );

    let mut led_rx = pins.led_rx.into_output(&mut pins.ddr);
    let mut led_tx = pins.led_tx.into_output(&mut pins.ddr);

    // // flash led
    //
    // for i in 0..=2 {
    //     arduino_leonardo::delay_ms(200);
    //     led_rx.set_low().expect("can't happen");
    //
    //     arduino_leonardo::delay_ms(200);
    //     led_rx.set_high().expect("can't happen");
    // }
    // arduino_leonardo::delay_ms(1000);

    // // flash backlight
    //
    // for i in 0..=2 {
    //     // light on
    //     write_i2c_extender(&mut i2c, 0x08);
    //     arduino_leonardo::delay_ms(200);
    //
    //     // light off
    //     write_i2c_extender(&mut i2c, 0x00);
    //     arduino_leonardo::delay_ms(200);
    // }

    // initialize display
    lcd_init(&mut i2c);

    // clear screen
    lcd_clear(&mut i2c);

    lcd_displaycontrol(&mut i2c, true, false, false);

    lcd_set_position(&mut i2c, 0, 0);
    lcd_write_string(&mut i2c, "Hallo Eisbaer...");

    arduino_leonardo::delay_ms(2000);

    lcd_set_position(&mut i2c, 1, 0);
    lcd_write_string(&mut i2c, "mit Fahrrad!");

    arduino_leonardo::delay_ms(2000);

    lcd_write_string(&mut i2c, " :-)");

    loop {
        led_rx.set_high().expect("can't happen");
        arduino_leonardo::delay_ms(500);

        led_tx.set_high().expect("can't happen");
        arduino_leonardo::delay_ms(500);

        led_rx.set_low().expect("can't happen");
        arduino_leonardo::delay_ms(500);

        led_tx.set_low().expect("can't happen");
        arduino_leonardo::delay_ms(500);
    }
}
