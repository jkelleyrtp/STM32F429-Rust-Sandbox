#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_semihosting;
pub use cortex_m_rt::entry;
use hal::{
    gpio::Output,
    gpio::{
        gpiog::{PG13, PG14},
        PushPull,
    },
    prelude::*,
};
use stm32f4::stm32f429 as stm32;
use stm32f4xx_hal as hal;

/// Layout how we'll use the pins on the stm32
/// https://www.st.com/resource/en/user_manual/dm00093903-discovery-kit-with-stm32f429zi-mcu-stmicroelectronics.pdf
struct BlinkConfig {
    // Configure the green led on the PG13 pin, in output mode, configured as push/pull
    led_green: PG13<Output<PushPull>>,
    led_red: PG14<Output<PushPull>>,

    // Bind the delay method to the chip's clock
    delay: hal::delay::Delay,
}

impl BlinkConfig {
    fn setup() -> Self {
        // Create and consume peripherals to create the program interface
        let (dp, cp) = (
            stm32::Peripherals::take().unwrap(),
            cortex_m::peripheral::Peripherals::take().unwrap(),
        );

        // Set up the LED.
        // On the STM32F429, the red led is on pg14, and the green on pg13
        let gpio = dp.GPIOG.split();

        // Set up the system clock. We want to run at 48MHz for this one.
        // We do this to create a delay abstraction based on SysTick
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        Self {
            led_green: gpio.pg13.into_push_pull_output(),
            led_red: gpio.pg14.into_push_pull_output(),
            delay: hal::delay::Delay::new(cp.SYST, clocks),
        }
    }
}

#[entry]
fn main() -> ! {
    let BlinkConfig {
        mut delay,
        mut led_green,
        mut led_red,
    } = BlinkConfig::setup();

    loop {
        // On for 1s, off for 1s.
        led_red.set_high().unwrap();
        led_green.set_low().unwrap();
        delay.delay_ms(1000_u32);

        led_green.set_high().unwrap();
        led_red.set_low().unwrap();
        delay.delay_ms(1000_u32);
    }
}
