# stm32F429 Discovery Board - Blinky in Rust


This project is me figuring out how to set up a simple embedded project with rust. I have this board:
https://www.st.com/resource/en/user_manual/dm00093903-discovery-kit-with-stm32f429zi-mcu-stmicroelectronics.pdf


I'm playing with a few different approaches of setting up new embedded projects and I quite like directly using Rust's type system as the layout configuration. Manifested for the blinky program:

```rust
struct BlinkConfig {
    // Configure the green led on the PG13 pin, in output mode, configured as push/pull
    led_green: PG13<Output<PushPull>>,
    led_red: PG14<Output<PushPull>>,

    // Bind the delay method to the chip's clock
    delay: hal::delay::Delay,
}
```

The BlinkConfig has a `setup` method which creates and consumes adapters for the board's peripherals. This cleans up the main loop and encourages the Rust ownership model of pins after setup. With this approach, we can also have platform-independent programs with different BlinkConfigurations depending on the platform. With the above method, this is the main loop for the blink program:

```rust
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
```

Clean, right? We were able to leverage the type system as our configuration mapping and the owernship model for platform-indenpdent programs.


## Running

I know there are better ways of configuring projects, etc.

Spin up openocd:
```
// In a terminal

cd /tmp
openocd -f interface/stlink-v2-1.cfg -f target/stm32f4x.cfg
```

Flash the chip
```
// In another

cargo run
> (gdb) target remote :3333
> (gdb) load
```

My programs only run when I hit the user button on the board, not sure what's up with that.
