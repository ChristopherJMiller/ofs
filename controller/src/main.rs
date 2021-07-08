#![no_std]
#![no_main]

use panic_halt as _;
use avr_device::atmega328p::Peripherals;

#[avr_device::entry]
fn main() -> ! {
  let peripherals = Peripherals::take().unwrap();

  let portb = &*peripherals.PORTB;
  portb.ddrb.write(|w| w.pb5().set_bit());
  portb.portb.write(|w| w.pb5().bit(true));

  loop {}
}
