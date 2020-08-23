#![no_main]
#![no_std]
pub use cortex_m_rt::entry;
use embedded as _; // global logger + panicking-behavior + memory layout
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
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

/// Layout how we'll use the pins on the stm32
/// https://www.st.com/resource/en/user_manual/dm00093903-discovery-kit-with-stm32f429zi-mcu-stmicroelectronics.pdf
struct BlinkConfig {
    // Configure the green led on the PG13 pin, in output mode, configured as push/pull
    led_green: PG13<Output<PushPull>>,
    led_red: PG14<Output<PushPull>>,

    // Bind the delay method to the chip's clock
    delay: hal::delay::Delay,

    usb_bus: UsbBusAllocator<UsbBus<USB>>,
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
        let gpioa = dp.GPIOA.split();

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

        let usb = USB {
            usb_global: dp.OTG_FS_GLOBAL,
            usb_device: dp.OTG_FS_DEVICE,
            usb_pwrclk: dp.OTG_FS_PWRCLK,
            pin_dm: gpioa.pa11.into_alternate_af10(),
            pin_dp: gpioa.pa12.into_alternate_af10(),
        };

        let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

        Self {
            led_green: gpiog.pg13.into_push_pull_output(),
            led_red: gpiog.pg14.into_push_pull_output(),
            delay: hal::delay::Delay::new(cp.SYST, clocks),
            usb_bus,
        }
    }
}

fn create_usb(
    usb_bus: &UsbBusAllocator<UsbBus<USB>>,
) -> (SerialPort<UsbBus<USB>>, UsbDevice<UsbBus<USB>>) {
    defmt::info!("Building");

    let serial = usbd_serial::SerialPort::new(usb_bus);

    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27db))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    (serial, usb_dev)
}

#[entry]
fn main() -> ! {
    let BlinkConfig {
        mut delay,
        mut led_green,
        mut led_red,
        usb_bus,
    } = BlinkConfig::setup();

    let (mut serial, mut usb_dev) = create_usb(&usb_bus);

    // Blink hello
    for _ in 0..10 {
        led_red.set_high();
        led_green.set_high();
        delay.delay_ms(50_u16);
        led_red.set_low();
        led_green.set_low();
        delay.delay_ms(50_u16);
    }

    loop {
        // On for 1s, off for 1s.
        led_green.set_low().unwrap();

        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }
        defmt::info!("Connection established");
        led_green.set_high().unwrap();

        let mut buf = [0u8; 64];

        if let Ok(count) = serial.read(&mut buf) {
            if count > 0 {
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        // Uppercase the value in the buffer
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
