#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use embassy_rp::gpio::AnyPin;
use embassy_rp::peripherals::PIN_18;
use embassy_rp::peripherals::PIN_19;
use embassy_rp::usb::Out;
use static_cell::StaticCell;
use cortex_m::peripheral;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::Peripheral;
use embassy_rp::gpio;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C1;
use embassy_rp::peripherals::{PWM_CH1, PIN_3, PWM_CH2, PIN_4};
use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;
use gpio::{Level, Output};
mod drivers;
use drivers::hygro::Hygro;
use embassy_rp::pwm;
use fixed::traits::ToFixed;

// use embassy_rp::gpio;
// use embassy_time::{Duration, Timer};
// use gpio::{Level, Output, AnyPin};
use {defmt_rtt as _, panic_probe as _};

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello, world!");
    let p = embassy_rp::init(Default::default());

    info!("It's about to break!");
    // let mut pw = singleton!(pwm::Pwm::new_output_b(p.PWM_CH1, p.PIN_3, c.clone()));

    let pw = p.PWM_CH1;
    let pn = p.PIN_3;
    let _led_h = spawner.spawn(blink_led(pw, pn)).unwrap();

    let pw2 = p.PWM_CH2;
    let pn2 = p.PIN_4;
    let _led_h = spawner.spawn(blink_led2(pw2, pn2)).unwrap();   
    // let mut led_on = Output::new(p.PIN_2, Level::Low);
    info!("Time to spawn a temp reader");
    let sda = p.PIN_18;
    let scl = p.PIN_19;
    let i2c1 = p.I2C1;

    spawner.spawn(read_temp(i2c1, sda, scl)).unwrap();

    // loop {
    //     Timer::after(Duration::from_millis(3000)).await;
    // }

}

#[embassy_executor::task]
async fn blink_led(pwm_ch1: PWM_CH1, pin3: PIN_3) {
// async fn blink_led(pwm: &'static mut pwm::Pwm<'static, PWM_CH1>, pin: AnyPin) {

    let mut c: pwm::Config = Default::default();
    let mut pw = pwm::Pwm::new_output_b(pwm_ch1, pin3, c.clone());
    // pwm.set_config(&c);
    c.top = 12000; //0x8000;
    c.divider = 240u16.to_fixed(); // Seems like this is implmented with a u8 as the largest divider
    c.compare_b = 8;

    // let mut led = Output::new(pin, Level::Low);
    // led.set_low();

    info!("Done setting up, running loop");
    let mut going_up: bool = true;
    let mut fade: u16 = 0;
    loop {
        // LEd PWM
        if going_up {
            fade+=1;
            if fade == 255 {
                going_up = false;
            }
        } else {
            fade-=1;
            if fade == 0 {
                going_up = true;
            }
        }

        c.compare_b = (fade as f32 * fade as f32 * 12000.0f32 / 65535.0f32) as u16;

        // c.compare_b = c.compare_b.rotate_left(1);
        // c.divider;
        pw.set_config(&c);
        Timer::after(Duration::from_millis(10)).await;
    }
}


#[embassy_executor::task]
async fn blink_led2(pwm_ch2: PWM_CH2, pin4: PIN_4) {
// async fn blink_led(pwm: &'static mut pwm::Pwm<'static, PWM_CH1>, pin: AnyPin) {

    let mut c: pwm::Config = Default::default();
    let mut pw = pwm::Pwm::new_output_a(pwm_ch2, pin4, c.clone());
    // pwm.set_config(&c);
    c.top = 4096; //0x8000;
    c.divider = 240u16.to_fixed(); // Seems like this is implmented with a u8 as the largest divider
    c.compare_a = 8;

    // let mut led = Output::new(pin, Level::Low);
    // led.set_low();

    info!("Done setting up, running loop");
    let mut going_up: bool = true;
    let mut fade: u16 = 0;
    loop {
        // LEd PWM
        if going_up {
            fade+=1;
            if fade == 255 {
                going_up = false;
            }
        } else {
            fade-=1;
            if fade == 0 {
                going_up = true;
            }
        }

        c.compare_a = (fade as f32 * fade as f32 * 4096.0f32 / 65535.0f32) as u16;

        // c.compare_b = c.compare_b.rotate_left(1);
        // c.divider;
        pw.set_config(&c);
        Timer::after(Duration::from_millis(5)).await;
    }
}

#[embassy_executor::task]
async fn read_temp(i2c1: I2C1, sda: PIN_18, scl: PIN_19) {

    info!("set up i2c ");
    let i2c = i2c::I2c::new_async(i2c1, scl, sda, Irqs, Config::default());

    let mut readings = [0f32; 2];
    // Setup i2c:
    let mut sensor = Hygro::new(i2c);
    sensor.init().await;

    loop {
        readings[0] = sensor.temperature().await;
        readings[1] = sensor.humidity().await;
        info!("temp: {} C, humidity: {}%", readings[0], readings[1]);

        Timer::after(Duration::from_millis(3000)).await;
    }

}
