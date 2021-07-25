#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use core::cell::RefCell;

use avr_device::atmega328p::{portb, Peripherals, PORTB, TC1};
use avr_device::interrupt::Mutex;
use avr_device::{entry, interrupt};
use fightstick::{build_fightstick_data, setup_ports};
use ofs_support::fightstick::{FightstickDescriptor, IDLE_FIGHTSTICK};
use ofs_support::usart::UsartCommand;
use panic_halt as _;
use support::alloc::ALLOCATOR;
use support::serial::{BAUD_9600, SERIAL};

pub mod fightstick;
pub mod support;

static G_PORTB: Mutex<RefCell<Option<PORTB>>> = Mutex::new(RefCell::new(None));
static G_TC1: Mutex<RefCell<Option<TC1>>> = Mutex::new(RefCell::new(None));
static QUEUE: Mutex<RefCell<Option<Vec<u8>>>> = Mutex::new(RefCell::new(None));
static FIGHTSTICK: Mutex<RefCell<FightstickDescriptor>> = Mutex::new(RefCell::new(IDLE_FIGHTSTICK));

fn configure_portb(portb: &portb::RegisterBlock) {
  portb.ddrb.modify(|_, w| w.pb5().set_bit());
  portb.portb.write(|w| w.pb5().clear_bit());
}

fn sei() {
  unsafe {
    interrupt::enable();
  }
}

fn configure_timer(tc1: &TC1) {
  tc1.ocr1a.write(|w| unsafe { w.bits(500) });
  tc1.tcnt1.write(|w| unsafe { w.bits(0) });
  tc1.timsk1.write(|w| w.ocie1a().set_bit());
  tc1.tccr1b.write(|w| w.cs1().prescale_1024());
}

#[entry]
fn main() -> ! {
  ALLOCATOR.init(0x500, 0x1FF);
  let peripherals = Peripherals::take().unwrap();

  configure_portb(&*peripherals.PORTB);

  interrupt::free(|cs| {
    QUEUE.borrow(cs).replace(Some(Vec::new()));
    G_PORTB.borrow(cs).replace(Some(peripherals.PORTB));

    // Configure Serial Singleton (USART0)
    SERIAL
      .borrow(cs)
      .borrow_mut()
      .setup(cs, peripherals.USART0, &peripherals.PORTD);
    SERIAL.borrow(cs).borrow().configure_uart(cs, BAUD_9600);

    setup_ports(cs, peripherals.PORTD);

    configure_timer(&peripherals.TC1);
    G_TC1.borrow(cs).replace(Some(peripherals.TC1));
  });

  sei();

  // Flush Transmission
  interrupt::free(|cs| {
    if let Ok(mut serial) = SERIAL.borrow(cs).try_borrow_mut() {
      serial.queue_many(cs, |serial| {
        for &i in [1, 2, 3, 4, 5, 6, 7, 8].iter() {
          serial.write(i as u8);
        }
      });
    }
  });

  loop {}
}

#[interrupt(atmega328p)]
fn TIMER1_COMPA() {
  interrupt::free(|cs| {
    let tc1 = G_TC1.borrow(cs).borrow();
    tc1.as_ref().unwrap().tccr1b.write(|w| w.cs1().no_clock());

    if let Ok(mut fightstick) = FIGHTSTICK.borrow(cs).try_borrow_mut() {
      *fightstick = build_fightstick_data(cs).into();
    }

    tc1.as_ref().unwrap().tcnt1.write(|w| unsafe { w.bits(0) });
    tc1.as_ref().unwrap().tccr1b.write(|w| w.cs1().prescale_1024());
  });
}

#[interrupt(atmega328p)]
fn USART_RX() {
  interrupt::free(|cs| {
    let command: UsartCommand = SERIAL.borrow(cs).borrow().read(cs).into();
    match command {
      UsartCommand::Introduction => {
        let portb = G_PORTB.borrow(cs).borrow();
        portb.as_ref().unwrap().portb.write(|w| w.pb5().set_bit());
        if let Ok(mut serial) = SERIAL.borrow(cs).try_borrow_mut() {
          serial.write_and_queue(cs, UsartCommand::Introduction.into());
        }
      },
      UsartCommand::SendData => {
        if let Ok(mut serial) = SERIAL.borrow(cs).try_borrow_mut() {
          if let Ok(fightstick) = FIGHTSTICK.borrow(cs).try_borrow() {
            let portb = G_PORTB.borrow(cs).borrow();
            portb.as_ref().unwrap().portb.modify(|r, w| w.pb5().bit(!r.pb5().bit()));
            serial.queue_many(cs, |serial| {
              serial.write(UsartCommand::SendData.into());
              serial.write(fightstick.0[0]);
              serial.write(fightstick.0[1]);
              serial.write(fightstick.0[2]);
              serial.write(fightstick.0[3]);
            });
          }
        }
      },
      _ => {}, // noop
    }
  });
}
