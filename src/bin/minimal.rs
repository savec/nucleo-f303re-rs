#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use rtic_monotonics::systick::*;
use stm32f3xx_hal::gpio::*;
use stm32f3xx_hal::prelude::*;
use test_app as _; // global logger + panicking-behavior + memory layout

#[rtic::app(
    device = stm32f3xx_hal::pac,
    dispatchers = [CAN_RX1],
    peripherals = true
)]
mod app {
    use super::*;
    // Shared resources go here
    #[shared]
    struct Shared {}

    // Local resources go here
    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        defmt::info!("init");

        // TODO setup monotonic if used
        let sysclk = { 36_000_000 };
        let token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, sysclk, token);

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let _clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(36.MHz())
            .pclk1(36.MHz())
            .freeze(&mut flash.acr);

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb);
        let mut led = gpioa
            .pa5
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        led.set_high().unwrap();
        blinker_task::spawn().ok();

        (Shared {}, Local { led })
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        defmt::info!("idle");

        loop {
            continue;
        }
    }

    #[task(priority = 1, local = [led])]
    async fn blinker_task(cx: blinker_task::Context) {
        loop {
            cx.local.led.toggle().unwrap();
            Systick::delay(500.millis()).await;

            defmt::info!("blink!");
        }
    }
}
