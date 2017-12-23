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

#![deny(unsafe_code)]
#![feature(proc_macro)]
#![no_std]

extern crate f3;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting as semihosting;

use core::fmt::Write;

use rtfm::{app, Threshold};
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
            resources: [LED_INDEX],
        },
    },
}

fn init(p: init::Peripherals, _r: init::Resources) {
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

fn toggle(_t: &mut Threshold, r: SYS_TICK::Resources) {
    LEDS[**r.LED_INDEX].off();
    **r.LED_INDEX += 1;
    if **r.LED_INDEX >= LEDS.len() {
        **r.LED_INDEX = 0;
    }
    LEDS[**r.LED_INDEX].on();
}
