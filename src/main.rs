// Copyright 2017 Clément Bizeau
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(proc_macro)]
#![no_std]

extern crate f3;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting as semihosting;

use core::fmt::Write;

use rtfm::{app, Threshold, Resource};
use semihosting::hio;
use cortex_m::peripheral::SystClkSource;
use f3::led::{self, LEDS};

app! {
    device: f3::stm32f30x,

    resources: {
        static LED_INDEX: usize = 0;
    },

    tasks: {
        SYS_TICK: {
            path: toggle,
            priority: 1,
            resources: [LED_INDEX],
        },
        EXTI0: {
            path: button,
            priority: 2,
            resources: [EXTI, LED_INDEX],
        },
    },
}

fn init(p: init::Peripherals, _r: init::Resources) {
    // configure interrupt for user button
    unsafe { p.SYSCFG.exticr1.modify(|_, w| w.exti0().bits(0)); }
    p.RCC.ahbenr.modify(|_, w| w.iopaen().enabled());
    p.EXTI.imr1.modify(|_, w| w.mr0().set_bit());
    p.EXTI.rtsr1.modify(|_, w| w.tr0().set_bit());

    led::init(p.GPIOE, p.RCC);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000 / 4);
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

fn idle() -> ! {
    writeln!(hio::hstdout().unwrap(), "Hello, world!").unwrap();

    loop {
        rtfm::wfi();
    }
}

/// Enable LED sequentially
fn toggle(t: &mut Threshold, SYS_TICK::Resources {mut LED_INDEX}: SYS_TICK::Resources) {
    LED_INDEX.claim_mut(t, |led_index, _t| {
        LEDS[**led_index].off();
        **led_index = (**led_index + 1) % LEDS.len();
        LEDS[**led_index].on();
    });
}

/// Button interrupt
///
/// Reset the LED sequence
fn button(t: &mut Threshold, EXTI0::Resources{EXTI, LED_INDEX}: EXTI0::Resources) {
    rtfm::atomic(t, |t| {
        let exti_reg = EXTI.borrow(t);
        let led_index = LED_INDEX.borrow_mut(t);

        exti_reg.pr1.modify(|_, w| w.pr0().set_bit());
        LEDS[**led_index].off();
        **led_index = 0;
    });
}
