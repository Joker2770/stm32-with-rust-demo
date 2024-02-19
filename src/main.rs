#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use core::fmt::Write; // for pretty formatting of the output
use cortex_m_rt::entry;
use mcp9808::{
    address::SlaveAddress,
    reg_conf::*,
    reg_res::*,
    reg_temp_generic::{ReadableTempRegister, WritableTempRegister},
    MCP9808,
};
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{i2c::Mode, pac, prelude::*};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    rtt_init_print!();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let rcc = dp.RCC.constrain();
    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();
    // Acquire the GPIOA peripheral
    let gpioa = dp.GPIOA.split();
    // Acquire the GPIOB peripheral
    let gpiob = dp.GPIOB.split();

    // define RX/TX pins
    let tx_pin = gpioa.pa2;
    // configure serial
    // let mut tx = Serial::tx(dp.USART1, tx_pin, 115200.bps(), &clocks).unwrap();
    // or
    let mut tx = dp.USART2.tx(tx_pin, 115200.bps(), &clocks).unwrap();

    let scl = gpiob.pb6.into_alternate_open_drain();
    let sda = gpiob.pb7.into_alternate_open_drain();

    let m_i2c1 = dp.I2C1.i2c(
        (scl, sda),
        Mode::Standard {
            frequency: 400_000_u32.Hz(),
        },
        &clocks,
    );

    // let mut buf = [0x05];
    // m_i2c1.write_read(0x18, &buf.clone(), &mut buf).unwrap();

    let mut mcp9808 = MCP9808::new(
        m_i2c1,
        SlaveAddress::Alternative {
            a2: false,
            a1: false,
            a0: false,
        },
    );

    // how to read & write register
    let mut conf = mcp9808.read_configuration().unwrap();
    conf.set_shutdown_mode(ShutdownMode::Continuous);
    let _c = mcp9808.write_register(conf);

    let mut alert_upper = mcp9808.read_alert_upper().unwrap();
    alert_upper.set_celsius(60.0 as f32);
    let _a_up = mcp9808.write_register(alert_upper);

    let mut alert_lower = mcp9808.read_alert_lower().unwrap();
    alert_lower.set_celsius(5.0 as f32);
    let _a_up = mcp9808.write_register(alert_lower);

    let mut delay = dp.TIM1.delay_ms(&clocks);
    loop {
        // read temperature register
        let temp = mcp9808
            .read_temperature()
            .unwrap()
            .get_celsius(ResolutionVal::Deg_0_0625C);
        rprintln!(
            "->temp: {:.4}, alert_up: {:.4}, alert_lower: {:.4}",
            temp,
            alert_upper.get_celsius(ResolutionVal::Deg_0_0625C),
            alert_lower.get_celsius(ResolutionVal::Deg_0_0625C)
        );

        let _ = writeln!(tx, "temperature: {}", temp);
        delay.delay(250.millis());
    }
}
