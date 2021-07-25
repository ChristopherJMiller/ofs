#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::cell::RefCell;

use avr_device::asm::wdr;
use avr_device::atmega8u2::{Peripherals, CPU, TC0, TC1};
use avr_device::interrupt::{enable, free, CriticalSection, Mutex};
use avr_device::{entry, interrupt};
use panic_halt as _;
use usart::{ask_for_fighstick_data, handshake_controller, setup_usart};
use usb::{send_gamepad_data, setup_usb};

pub mod descriptors;
pub mod usart;
pub mod usb;

static G_TC0: Mutex<RefCell<Option<TC0>>> = Mutex::new(RefCell::new(None));
static G_TC1: Mutex<RefCell<Option<TC1>>> = Mutex::new(RefCell::new(None));

fn sei() {
  unsafe {
    enable();
  }
}

fn setup_cpu(_: &CriticalSection, cpu: CPU) {
  cpu.mcusr.write(|w| unsafe { w.bits(0) });
  wdr();
  cpu.clkpr.write(|w| w.clkpce().set_bit());
  cpu.clkpr.write(|w| w.clkps().bits(0));
}

fn configure_timer(cs: &CriticalSection) {
  let tc0 = G_TC0.borrow(cs).borrow();

  tc0.as_ref().unwrap().ocr0a.write(|w| unsafe { w.bits(255) });
  tc0.as_ref().unwrap().tcnt0.write(|w| unsafe { w.bits(0) });
  tc0.as_ref().unwrap().timsk0.write(|w| w.ocie0a().set_bit());
  tc0.as_ref().unwrap().tccr0b.write(|w| w.cs0().prescale_1024());
}

fn configure_usb_startup_delay(tc1: &TC1) {
  tc1.ocr1a.write(|w| unsafe { w.bits(60000) });
  tc1.tcnt1.write(|w| unsafe { w.bits(0) });
  tc1.timsk1.write(|w| w.ocie1a().set_bit());
  tc1.tccr1b.write(|w| w.cs1().prescale_1024());
}

#[entry]
fn main() -> ! {
  let peripherals = Peripherals::take().unwrap();

  free(|cs| {
    setup_usart(cs, peripherals.USART1, &peripherals.PORTD);
    setup_cpu(cs, peripherals.CPU);
    setup_usb(cs, peripherals.USB_DEVICE, peripherals.PLL, peripherals.PORTD);
    configure_usb_startup_delay(&peripherals.TC1);

    G_TC0.borrow(cs).replace(Some(peripherals.TC0));
    G_TC1.borrow(cs).replace(Some(peripherals.TC1));
  });

  sei();

  loop {}
}

#[interrupt(atmega8u2)]
fn TIMER1_COMPA() {
  interrupt::free(|cs| {
    configure_timer(cs);
    handshake_controller(cs);
    let tc1 = G_TC1.borrow(cs).borrow();
    tc1.as_ref().unwrap().tccr1b.write(|w| w.cs1().no_clock());
  });
}

#[interrupt(atmega8u2)]
fn TIMER0_COMPA() {
  interrupt::free(|cs| {
    let tc0 = G_TC0.borrow(cs).borrow();

    tc0.as_ref().unwrap().tccr0b.write(|w| w.cs0().no_clock());

    ask_for_fighstick_data(cs);
    send_gamepad_data(cs);

    tc0.as_ref().unwrap().tcnt0.write(|w| unsafe { w.bits(0) });
    tc0.as_ref().unwrap().tccr0b.write(|w| w.cs0().prescale_1024());
  });
}
