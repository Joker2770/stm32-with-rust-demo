#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use core::fmt::Write; // for pretty formatting of the output
use cortex_m_rt::entry;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use stm32f1xx_hal::{
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    // let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();
    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);

    // Acquire the GPIOB peripheral
    let mut gpiob = dp.GPIOB.split();

    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let _m_i2c1 = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400.kHz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        1000,
        10,
        1000,
        1000,
    );

    let interface = I2CDisplayInterface::new(_m_i2c1);

    let mut display =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();
    display.init().unwrap();

    let mut delay = dp.TIM1.delay_ms(&clocks);
    loop {
        display.clear().unwrap();
        // Spam some characters to the display
        for c in 0..3 {
            let _ = writeln!(display, "{}", c);

            delay.delay(100.millis());
        }

        // The `write!()` macro is also supported
        let _ = writeln!(display, "Hello, {}", "world");

        delay.delay(250.millis());
    }
}
