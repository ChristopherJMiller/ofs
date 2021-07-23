
use avr_device::atmega8u2::USART1;
use avr_device::interrupt::{Mutex, CriticalSection};
use core::cell::RefCell;

pub static USART: Mutex<RefCell<Option<USART1>>> = Mutex::new(RefCell::new(None));

pub fn setup_usart(cs: &CriticalSection, usart: USART1) {
  usart.ubrr1.write(|w| unsafe { w.bits(25) });
  usart.ucsr1b.write(|w| w.rxen1().set_bit().txen1().set_bit());
  usart.ucsr1c.write(|w| w.ucsz1().chr8());
}
