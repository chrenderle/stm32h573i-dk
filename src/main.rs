#![no_std]
#![no_main]

use core::future::pending;

use defmt::*;
use display_interface_parallel_gpio::{Generic16BitBus, PGPIO16BitInterface};
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, gpio::{Level, Output, Speed}, peripherals};
use embassy_time::Timer;
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor, draw_target::DrawTarget};
use mipidsi::{models::ST7789, options::{ColorInversion, RefreshOrder}, Builder};
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_WIDTH: usize = 240;
const DISPLAY_HEIGHT: usize = 240;
pub type TargetPixelType = u8;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let user_led1 = p.PI9;
    let user_led2 = p.PI8;
    let user_led3 = p.PF1;
    let user_led4 = p.PF4;
    
    let lcd_te = p.PD3;
    let lcd_d15 = p.PD10;
    let lcd_d14 = p.PD9;
    let lcd_d13 = p.PD8;
    let lcd_d12 = p.PE15;
    let lcd_d11 = p.PE14;
    let lcd_d10 = p.PE13;
    let lcd_d9 = p.PE12;
    let lcd_d8 = p.PE11;
    let lcd_d7 = p.PE10;
    let lcd_d6 = p.PE9;
    let lcd_d5 = p.PE8;
    let lcd_d4 = p.PE7;
    let lcd_d3 = p.PD1;
    let lcd_d2 = p.PD0;
    let lcd_d1 = p.PD15;    
    let lcd_d0 = p.PD14;
    let lcd_noe = p.PD4;
    let lcd_nwe = p.PD5;
    let lcd_a0_rs = p.PF0;
    let lcd_ne1_cs = p.PC7;
    let lcd_rst = p.PH13;
    let lcd_bl_ctrl = p.PI3;
    let power = p.PC6;


    // led task
    let led = Output::new(user_led1, Level::Low, Speed::Low);
    spawner.spawn(blinky(led)).unwrap();
    
    
    // display task
    let initial_level = Level::Low;
    let speed = Speed::Medium;
    let lcd_d0 = Output::new(lcd_d0, initial_level, speed);
    let lcd_d1 = Output::new(lcd_d1, initial_level, speed);
    let lcd_d2 = Output::new(lcd_d2, initial_level, speed);
    let lcd_d3 = Output::new(lcd_d3, initial_level, speed);
    let lcd_d4 = Output::new(lcd_d4, initial_level, speed);
    let lcd_d5 = Output::new(lcd_d5, initial_level, speed);
    let lcd_d6 = Output::new(lcd_d6, initial_level, speed);
    let lcd_d7 = Output::new(lcd_d7, initial_level, speed);
    let lcd_d8 = Output::new(lcd_d8, initial_level, speed);
    let lcd_d9 = Output::new(lcd_d9, initial_level, speed);
    let lcd_d10 = Output::new(lcd_d10, initial_level, speed);
    let lcd_d11 = Output::new(lcd_d11, initial_level, speed);
    let lcd_d12 = Output::new(lcd_d12, initial_level, speed);
    let lcd_d13 = Output::new(lcd_d13, initial_level, speed);
    let lcd_d14 = Output::new(lcd_d14, initial_level, speed);
    let lcd_d15 = Output::new(lcd_d15, initial_level, speed);
    let lcd_wr = Output::new(lcd_nwe, initial_level, speed);
    let lcd_dc = Output::new(lcd_a0_rs, initial_level, speed);
    let lcd_rst = Output::new(lcd_rst, initial_level, speed);
    let backlight = Output::new(lcd_bl_ctrl, initial_level, speed);
    let power = Output::new(power, initial_level, speed);
    let lcd_cs = Output::new(lcd_ne1_cs, initial_level, speed);
    spawner.spawn(display_task(DisplayTaskArguments {
        lcd_d0,
        lcd_d1,
        lcd_d2,
        lcd_d3,
        lcd_d4,
        lcd_d5,
        lcd_d6,
        lcd_d7,
        lcd_d8,
        lcd_d9,
        lcd_d10,
        lcd_d11,
        lcd_d12,
        lcd_d13,
        lcd_d14,
        lcd_d15,
        lcd_wr,
        lcd_dc,
        lcd_rst,
        backlight,
        power,
        lcd_cs,
    })).unwrap();
}

#[embassy_executor::task]
async fn blinky(mut led: Output<'static>) -> ! {
    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }
}

struct DisplayTaskArguments<'a> {
    lcd_d0: Output<'a>,
    lcd_d1: Output<'a>,
    lcd_d2: Output<'a>,
    lcd_d3: Output<'a>,
    lcd_d4: Output<'a>,
    lcd_d5: Output<'a>,
    lcd_d6: Output<'a>,
    lcd_d7: Output<'a>,
    lcd_d8: Output<'a>,
    lcd_d9: Output<'a>,
    lcd_d10: Output<'a>,
    lcd_d11: Output<'a>,
    lcd_d12: Output<'a>,
    lcd_d13: Output<'a>,
    lcd_d14: Output<'a>,
    lcd_d15: Output<'a>,
    lcd_wr: Output<'a>,
    lcd_dc: Output<'a>,
    lcd_rst: Output<'a>,
    lcd_cs: Output<'a>,
    backlight: Output<'a>,
    power: Output<'a>,
}

#[embassy_executor::task]
async fn display_task(mut arguments: DisplayTaskArguments<'static>) -> ! {
    // enable power
    arguments.power.set_low();
    
    // enable chip select
    arguments.lcd_cs.set_low();
    
    Timer::after_millis(50).await;

    let bus = Generic16BitBus::new((arguments.lcd_d0, arguments.lcd_d1, arguments.lcd_d2, arguments.lcd_d3, arguments.lcd_d4, arguments.lcd_d5, arguments.lcd_d6, arguments.lcd_d7, arguments.lcd_d8, arguments.lcd_d9, arguments.lcd_d10, arguments.lcd_d11, arguments.lcd_d12, arguments.lcd_d13, arguments.lcd_d14, arguments.lcd_d15));
    let display_interface = PGPIO16BitInterface::new(bus, arguments.lcd_dc, arguments.lcd_wr);
    let mut delay = embassy_time::Delay;
    let mut display = Builder::new(ST7789, display_interface)
        .display_size(DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16)
        .color_order(mipidsi::options::ColorOrder::Rgb)
        .reset_pin(arguments.lcd_rst)
        .init(&mut delay).unwrap();
    
    // enable backlight
    arguments.backlight.set_high();
    
    display.clear(Rgb565::RED).unwrap();
    info!("drew display");
    loop {
        pending::<()>().await;
    }
}