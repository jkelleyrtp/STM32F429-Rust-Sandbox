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
use stm32f4xx_hal::otg_fs::{UsbBus, USB};

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
        let gpiog = dp.GPIOG.split();

        // Set up the system clock. We want to run at 48MHz for this one.
        // We do this to create a delay abstraction based on SysTick
        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .require_pll48clk()
            .freeze();
        Self {
            led_green: gpiog.pg13.into_push_pull_output(),
            led_red: gpiog.pg14.into_push_pull_output(),
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
        // Blink hello
        for _ in 0..10 {
            led_red.set_high();
            led_green.set_high();
            delay.delay_ms(50_u16);
            led_red.set_low();
            led_green.set_low();
            delay.delay_ms(50_u16);
        }
    }
}
