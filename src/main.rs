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
    info!("Starting up!");
    let p = embassy_rp::init(Default::default());

    let pw = p.PWM_CH1;
    let pn = p.PIN_3;
    let _led_h = spawner.spawn(pulse_led_1(pw, pn)).unwrap();

    let pw2 = p.PWM_CH2;
    let pn2 = p.PIN_4;
    let _led_h = spawner.spawn(pulse_led_2(pw2, pn2)).unwrap();
    info!("All tasks initialized");

}

/// LED pulse task 1.
/// 
/// This function is designed to not be generic to make it easier to adapt.
/// It takes a specific `PWM` channel and `PIN` combination, see Pico documentation.
/// 
#[embassy_executor::task]
async fn pulse_led_1(pwm_ch1: PWM_CH1, pin3: PIN_3) {
// async fn blink_led(pwm: &'static mut pwm::Pwm<'static, PWM_CH1>, pin: AnyPin) {

    let mut c: pwm::Config = Default::default();
    let mut pw = pwm::Pwm::new_output_b(pwm_ch1, pin3, c.clone());

    c.top = 12000; //0x8000;
    c.divider = 240u16.to_fixed(); // Seems like this is implmented with a u8 as the largest divider
    c.compare_b = 8;

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

        pw.set_config(&c);
        Timer::after(Duration::from_millis(10)).await;
    }
}


#[embassy_executor::task]
async fn pulse_led_2(pwm_ch2: PWM_CH2, pin4: PIN_4) {
// async fn blink_led(pwm: &'static mut pwm::Pwm<'static, PWM_CH1>, pin: AnyPin) {

    let mut c: pwm::Config = Default::default();
    let mut pw = pwm::Pwm::new_output_a(pwm_ch2, pin4, c.clone());
    // pwm.set_config(&c);
    c.top = 4096; //0x8000;
    c.divider = 240u16.to_fixed(); // Seems like this is implmented with a u8 as the largest divider
    c.compare_a = 8;

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

