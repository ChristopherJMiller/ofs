
use panic_halt as _;
use avr_device::atmega328p::{USART0, PORTD};
use avr_device::interrupt;
use avr_device::interrupt::{CriticalSection, Mutex};
use core::cell::RefCell;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::string::String;

pub static BAUD_9600: u16 = 207;

pub struct Serial {
  usart0: Mutex<RefCell<Option<USART0>>>,
  ready: bool,
  queue: Option<VecDeque<u8>>,
}

impl Serial {
  pub fn setup(&mut self, cs: &CriticalSection, usart0: USART0, portd: PORTD) {
    portd.ddrd.write(|w| w.pd0().set_bit().pd1().set_bit());
    self.usart0.borrow(cs).replace(Some(usart0));
    self.ready = true;
    self.queue = Some(VecDeque::new());
  }

  pub fn configure_uart(&self, cs: &CriticalSection, ubrrn: u16) {
    if self.ready {
      let usart0 = self.usart0.borrow(cs).borrow();
      usart0.as_ref().unwrap().ucsr0a.write(|w| w.u2x0().set_bit()); // Double Baud
      usart0.as_ref().unwrap().ubrr0.write(|w| unsafe { w.bits(ubrrn) });
      usart0.as_ref().unwrap().ucsr0c.write(|w| 
        w
        .umsel0().usart_async() // Async USART
        .upm0().disabled() // No Parity
        .usbs0().stop1() // 1 Stop Bit
        .ucsz0().chr8() // 8 Character Bits
      );
      usart0.as_ref().unwrap().ucsr0b.write(|w| w.txcie0().set_bit().txen0().set_bit()); // Enable TX and TX Interr
    }
  }

  pub fn write_text(&mut self, cs: &CriticalSection, text: String) {
    let chars = text.chars();
    let reversed = chars.map(|c| c as u8).collect::<Vec<u8>>();
    self.write(cs, &mut VecDeque::from(reversed));
  }

  pub fn write(&mut self, cs: &CriticalSection, data: &mut VecDeque<u8>) {
    if self.ready {
      self.queue.as_mut().unwrap().append(data);
      self.write_to_udr(cs);
    }
  }

  pub fn write_to_udr(&mut self, cs: &CriticalSection) {
    if let Some(c) = self.queue.as_mut().unwrap().pop_front() {
      let usart0 = self.usart0.borrow(cs).borrow();
      usart0.as_ref().unwrap().udr0.write(|w| unsafe { w.bits(c) });
    }
  }
}

#[interrupt(atmega328p)]
fn USART_TX() {
  interrupt::free(|cs| {
    SERIAL.borrow(cs).borrow_mut().write_to_udr(cs);
  });
}

pub static SERIAL: Mutex<RefCell<Serial>> = Mutex::new(RefCell::new(Serial {
  usart0: Mutex::new(RefCell::new(None)),
  ready: false,
  queue: None,
}));
