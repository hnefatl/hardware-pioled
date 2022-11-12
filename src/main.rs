#![no_std]
#![no_main]
#![feature(exhaustive_patterns)]
#![feature(stmt_expr_attributes)]

//use panic_halt as _; // breakpoint on `rust_begin_unwind` to catch panics
use panic_semihosting as _;

use core::convert::TryInto;
use cortex_m_rt::entry;
use embedded_hal::prelude::*;
use stm32f3xx_hal::{
    block, i2c, pac,
    prelude::*,
    time::duration::{Milliseconds, Seconds},
    timer::Timer,
};

//use pioled;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let mut reset_and_clock_control = peripherals.RCC.constrain();
    let mut flash = peripherals.FLASH.constrain();
    let clocks = reset_and_clock_control.cfgr.freeze(&mut flash.acr);
    let mut timer = Timer::new(peripherals.TIM1, clocks, &mut reset_and_clock_control.apb2);

    // For determining which bus (ahb) is needed, section 3.2.2 in
    // https://www.st.com/resource/en/reference_manual/dm00043574-stm32f303xb-c-d-e-stm32f303x6-8-stm32f328x8-stm32f358xc-stm32f398xe-advanced-arm-based-mcus-stmicroelectronics.pdf
    // documents which peripherals are reachable over which buses.
    let mut gpiob = peripherals.GPIOB.split(&mut reset_and_clock_control.ahb);

    let scl = gpiob
        .pb6
        .into_af_open_drain::<4>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let sda = gpiob
        .pb7
        .into_af_open_drain::<4>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    let i2c = i2c::I2c::new(
        peripherals.I2C1,
        (scl, sda),
        // Should support 100KHz, 400KHz, or 1MHz.
        100_u32.kHz().try_into().unwrap(),
        clocks,
        &mut reset_and_clock_control.apb1,
    );

    let i2c_interface = I2CDisplayInterface::new(i2c);

    let mut display =
        Ssd1306::new(i2c_interface, DisplaySize128x32, DisplayRotation::Rotate0).into_buffered_graphics_mode();
    display.init().unwrap();

    render_stuff(&mut display, &mut timer);
}

fn render_stuff<DI, SIZE, TIM>(
    display: &mut Ssd1306<DI, SIZE, ssd1306::mode::BufferedGraphicsMode<SIZE>>,
    timer: &mut Timer<TIM>,
) -> !
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
    Timer<TIM>: embedded_hal::timer::CountDown,
    Seconds: Into<<Timer<TIM> as embedded_hal::timer::CountDown>::Time>,
    Milliseconds: Into<<Timer<TIM> as embedded_hal::timer::CountDown>::Time>,
{
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Whoo, working!", Point::new(0, 0), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    display.flush().unwrap();
    timer.start(Seconds(2));
    block!(timer.wait()).unwrap();

    loop {
        display.clear();
        display.flush().unwrap();

        fill_screen(display, timer);
    }
}

fn fill_screen<DI, SIZE, TIM>(
    display: &mut Ssd1306<DI, SIZE, ssd1306::mode::BufferedGraphicsMode<SIZE>>,
    timer: &mut Timer<TIM>,
) where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
    Timer<TIM>: embedded_hal::timer::CountDown,
    Milliseconds: Into<<Timer<TIM> as embedded_hal::timer::CountDown>::Time>,
{
    let (width, height) = display.dimensions();
    for y in 0..height {
        for x in 0..width {
            display.set_pixel(x as u32, y as u32, true);
            display.flush().unwrap();
            timer.start(Milliseconds(1u32));
            block!(timer.wait()).unwrap();
        }
    }
}
