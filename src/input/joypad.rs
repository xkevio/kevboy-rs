use std::time::Duration;

use eframe::egui::{Context, Key};
use gilrs::{ev::filter::Repeat, Axis, Button, Event, EventType, Filter, Gilrs};
use hashlink::LinkedHashMap;

use crate::{
    cpu::interrupts::{Interrupt, InterruptHandler},
    mmu::mmio::MMIO,
};

#[derive(Debug, Clone, Copy)]
pub struct Joypad {
    joyp: u8,
    prev_joyp: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ButtonType {
    Action,
    Direction,
    None,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            joyp: 0xCF,
            prev_joyp: 0xCF,
        }
    }
}

impl MMIO for Joypad {
    fn read(&mut self, _address: u16) -> u8 {
        self.joyp
    }

    fn write(&mut self, _address: u16, value: u8) {
        self.reset_pressed_keys();
        self.joyp = 0xC0 | (value & 0x30) | (self.joyp & 0xF); // bit 7 and 6 unused and always 1
    }
}

impl Joypad {
    /// Handles input depending on the selected `ButtonType` (bits of the JOYP register)
    /// and looks out for possible Joypad IRQs.
    pub fn tick(
        &mut self,
        ctx: &Context,
        interrupt_handler: &mut InterruptHandler,
        action_keys: &LinkedHashMap<String, (Key, Button)>,
        direction_keys: &LinkedHashMap<String, (Key, Button)>,
        gilrs: &mut Gilrs,
    ) {
        match self.get_button_type() {
            ButtonType::Action => self.handle_key_input(ctx, action_keys, gilrs),
            ButtonType::Direction => self.handle_key_input(ctx, direction_keys, gilrs),
            _ => {}
        }

        // Joypad IRQ gets requested when (the lower 4 bits of) JOYP changes from 0xF to anything else.
        if (self.prev_joyp & 0xF == 0xF) && (self.joyp & 0xF != 0xF) {
            interrupt_handler.request_interrupt(Interrupt::Joypad);
        }

        self.prev_joyp = self.joyp;
    }

    /// Set all 4 key bits to 1 as that stands for "not pressed".
    pub fn reset_pressed_keys(&mut self) {
        self.joyp |= 0xF;
    }

    /// `key1`: **A** or **Right**
    ///
    /// `key2`: **B** or **Left**
    ///
    /// `key3`: **Select** or **Up**
    ///
    /// `key4`: **Start** or **Down**
    ///
    /// Takes `keys` from `ControlPanel` to ensure separation
    /// between UI and backend. Meaning, two extra references will
    /// have to be passed every tick.
    ///
    /// This implicit order is based on the bits of JOYP.
    fn handle_key_input(
        &mut self,
        ctx: &Context,
        keys: &LinkedHashMap<String, (Key, Button)>,
        gilrs: &mut Gilrs,
    ) {
        for (bit, (name, (key, button))) in keys.iter().enumerate() {
            if ctx.input(|i| i.key_down(*key)) || self.is_gamepad_button_down(gilrs, button, name) {
                self.joyp &= !(0x1 << bit as u8);
            }
        }
    }

    /// Checks if a controller button is pressed instead.
    ///
    /// Repeats the input as one input is too few for most games polling.
    fn is_gamepad_button_down(&self, gilrs: &mut Gilrs, button: &Button, name: &str) -> bool {
        let left_stick = gilrs.gamepads().any(|(_, g)| {
            if let (Some(axis_x), Some(axis_y)) =
                (g.axis_data(Axis::LeftStickX), g.axis_data(Axis::LeftStickY))
            {
                axis_x.value() > 0.5 && name == "Right"
                    || axis_x.value() < -0.5 && name == "Left"
                    || axis_y.value() > 0.5 && name == "Up"
                    || axis_y.value() < -0.5 && name == "Down"
            } else {
                false
            }
        });

        left_stick || gilrs
            .next_event()
            .filter_ev(
                &Repeat {
                    after: Duration::from_millis(0),
                    every: Duration::from_millis(10),
                },
                gilrs,
            )
            .map_or(false, |Event { event, .. }| {
                matches!(event, EventType::ButtonRepeated(b, _) if b == *button)
            })
    }

    fn get_button_type(&self) -> ButtonType {
        if self.joyp & 0x20 == 0 {
            ButtonType::Action
        } else if self.joyp & 0x10 == 0 {
            ButtonType::Direction
        } else {
            ButtonType::None
        }
    }
}
