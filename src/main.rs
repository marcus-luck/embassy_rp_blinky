#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C1;
use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;
use gpio::{Level, Output};
mod drivers;
use drivers::hygro::Hygro;
// use embassy_rp::gpio;
// use embassy_time::{Duration, Timer};
// use gpio::{Level, Output, AnyPin};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut led_on = Output::new(p.PIN_2, Level::Low);
    led_on.set_high();

    let sda = p.PIN_18;
    let scl = p.PIN_19;

    info!("set up i2c ");
    let i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

    let mut readings = [0f32; 2];
    // let sensor = HYGRO::Hygro::new(i2c, HYGRO::ADDR);
    // use HYGRO::*;
    // databus.set(cnt);

    // Setup i2c:
    let mut sensor = Hygro::new(i2c);
    sensor.init().await;

    info!("Done setting up, running loop");

    loop {

        readings[0] = sensor.temperature().await;
        readings[1] = sensor.humidity().await;

        info!("temp: {} C, humidity: {}%", readings[0], readings[1]);

        Timer::after(Duration::from_millis(10000)).await;
    }

}
