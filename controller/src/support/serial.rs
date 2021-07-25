
use panic_halt as _;
use avr_device::atmega328p::{USART0, PORTD};
use avr_device::interrupt;
use avr_device::interrupt::{CriticalSection, Mutex};
use core::cell::RefCell;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::string::String;

pub static BAUD_9600: u16 = 207;
pub static BAUD_38400: u16 = 51;

pub struct Serial {
  pub usart0: Mutex<RefCell<Option<USART0>>>,
  ready: bool,
  capacity: usize,
  queue: Option<VecDeque<u8>>,
}

impl Serial {
  pub fn setup(&mut self, cs: &CriticalSection, usart0: USART0, portd: &PORTD) {
    portd.ddrd.modify(|_, w| w.pd1().set_bit().pd0().clear_bit());
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
      usart0.as_ref().unwrap().ucsr0b.write(|w| 
        w.txcie0().set_bit()
        .rxcie0().set_bit()
        .rxen0().set_bit()
        .txen0().set_bit()
      );
    }
  }

  pub fn queued_size(&self) -> usize {
    self.queue.as_ref().unwrap().len()
  }

  pub fn space_available(&self) -> usize {
    self.capacity - self.queue.as_ref().unwrap().len()
  }

  pub fn queue_many<F>(&mut self, cs: &CriticalSection, f: F) where F: Fn(&mut Self) -> () {
    f(self);
    self.write_to_udr(cs);
  }

  pub fn write(&mut self, data: u8) -> bool {
    if let Some(queue) = self.queue.as_mut() {
      if self.ready && queue.len() < self.capacity {
        queue.push_back(data);
        return true
      }
    }
    false
  }

  pub fn write_and_queue(&mut self, cs: &CriticalSection, data: u8) {
    if self.write(data) {
      self.write_to_udr(cs);
    }
  }

  pub fn write_to_udr(&mut self, cs: &CriticalSection) {
    let usart0_borrow = self.usart0.borrow(cs).borrow();
    if let Some(usart0) = usart0_borrow.as_ref() {
      if usart0.ucsr0a.read().udre0().bit_is_set() {
        if let Some(m_queue) = self.queue.as_mut() {
          if let Some(c) = m_queue.pop_front() {
            usart0.udr0.write(|w| unsafe { w.bits(c) });
          }
        }
      }
    }
  }

  pub fn read(&self, cs: &CriticalSection) -> u8 {
    let usart0 = self.usart0.borrow(cs).borrow();
    usart0.as_ref().unwrap().udr0.read().bits()
  }
}

pub fn text_to_usart_vec(text: String) -> Vec<u8> {
  let chars = text.chars();
  return chars.map(|c| c as u8).collect::<Vec<u8>>();
}

#[interrupt(atmega328p)]
fn USART_TX() {
  interrupt::free(|cs| {
    if let Ok(mut serial) = SERIAL.borrow(cs).try_borrow_mut() {
      serial.write_to_udr(cs);
    }
  });
}

pub static SERIAL: Mutex<RefCell<Serial>> = Mutex::new(RefCell::new(Serial {
  usart0: Mutex::new(RefCell::new(None)),
  ready: false,
  capacity: 64,
  queue: None,
}));
