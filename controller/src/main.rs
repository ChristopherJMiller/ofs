#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use panic_halt as _;
use avr_device::atmega328p::{Peripherals, PORTB, TC1};
use avr_device::atmega328p::portb;
use avr_device::atmega328p::tc1;
use avr_device::{entry, interrupt};
use avr_device::interrupt::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::RefCell;
use support::serial::{BAUD_9600, SERIAL};
use support::alloc::ALLOCATOR;
use alloc::string::String;

pub mod support;

static G_PORTB: Mutex<RefCell<Option<PORTB>>> = Mutex::new(RefCell::new(None));
static G_TC1: Mutex<RefCell<Option<TC1>>> = Mutex::new(RefCell::new(None));
static STATE: AtomicBool = AtomicBool::new(false);

fn configure_timer(tc1: &tc1::RegisterBlock) {
  tc1.ocr1a.write(|w| unsafe { w.bits(10000) });
  tc1.tcnt1.write(|w| unsafe { w.bits(0) });
  tc1.timsk1.write(|w| w.ocie1a().set_bit());
  tc1.tccr1b.write(|w| w.cs1().prescale_1024());
}

fn configure_portb(portb: &portb::RegisterBlock) {
  portb.ddrb.write(|w| w.pb5().set_bit());
}

fn sei() {
  unsafe { interrupt::enable(); }
}

#[entry]
fn main() -> ! {
  ALLOCATOR.init(0x500, 0x1FF);
  let peripherals = Peripherals::take().unwrap();

  configure_timer(&*peripherals.TC1);
  configure_portb(&*peripherals.PORTB);

  interrupt::free(|cs| {
    G_PORTB.borrow(cs).replace(Some(peripherals.PORTB));
    G_TC1.borrow(cs).replace(Some(peripherals.TC1));

    // Configure Serial Singleton (USART0)
    SERIAL.borrow(cs).borrow_mut().setup(cs, peripherals.USART0, peripherals.PORTD);
    SERIAL.borrow(cs).borrow().configure_uart(cs, BAUD_9600);
  });

  sei();

  interrupt::free(|cs| {
    SERIAL.borrow(cs).borrow_mut().write(cs, String::from("Test"));
  });

  loop {}
}

#[interrupt(atmega328p)]
fn TIMER1_COMPA() {
  interrupt::free(|cs| {
    let portb = G_PORTB.borrow(cs).borrow();
    let value = STATE.load(Ordering::Relaxed);
    portb.as_ref().unwrap().portb.write(|w| w.pb5().bit(value));

    let tc1 = G_TC1.borrow(cs).borrow();
    tc1.as_ref().unwrap().tcnt1.write(|w| unsafe { w.bits(0) });

    STATE.store(!value, Ordering::Relaxed);
  });
}

