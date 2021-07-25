use core::cell::{Ref, RefCell};

use avr_device::atmega8u2::{PORTD, USART1};
use avr_device::interrupt;
use avr_device::interrupt::{CriticalSection, Mutex};
use ofs_support::fightstick::{FightstickDescriptor, IDLE_FIGHTSTICK};
use ofs_support::usart::UsartCommand;

static USART: Mutex<RefCell<Option<USART1>>> = Mutex::new(RefCell::new(None));
static SENT_INTRO: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static INTRO_COMPLETE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static GETTING_DATA: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static FIGHTSTICK_TABLE_POINTER: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));
static STAGING_FIGHTSTICK: Mutex<RefCell<FightstickDescriptor>> =
  Mutex::new(RefCell::new(FightstickDescriptor([0, 0, 0, 0])));
static FIGHTSTICK: Mutex<RefCell<FightstickDescriptor>> = Mutex::new(RefCell::new(IDLE_FIGHTSTICK));

pub fn setup_usart(cs: &CriticalSection, usart: USART1, portd: &PORTD) {
  usart.ubrr1.write(|w| unsafe { w.bits(103) });
  portd.ddrd.write(|w| w.pd2().clear_bit().pd3().set_bit());
  usart.ucsr1c.write(|w| {
    w.umsel1()
            .usart_async() // Async USART
            .upm1()
            .disabled() // No Parity
            .usbs1()
            .stop1() // 1 Stop Bit
            .ucsz1()
            .chr8()
  });
  usart
    .ucsr1b
    .write(|w| w.rxen1().set_bit().txen1().set_bit().rxcie1().set_bit());
  USART.borrow(cs).replace(Some(usart));
}

pub fn send_command(usart: &Ref<Option<USART1>>, command: UsartCommand) {
  send_data(usart, command.into());
}

pub fn send_data(usart: &Ref<Option<USART1>>, data: u8) {
  usart.as_ref().unwrap().udr1.write(|w| unsafe { w.bits(data) });
}

pub fn handshake_controller(cs: &CriticalSection) {
  if let Ok(mut sent_intro) = SENT_INTRO.borrow(cs).try_borrow_mut() {
    let usart = USART.borrow(cs).borrow();

    *sent_intro = true;
    send_command(&usart, UsartCommand::Introduction);
  }
}

pub fn get_fightstick_data(cs: &CriticalSection) -> FightstickDescriptor {
  FIGHTSTICK.borrow(cs).borrow().clone()
}

pub fn introduction_complete(cs: &CriticalSection) -> bool {
  *INTRO_COMPLETE.borrow(cs).borrow()
}

pub fn ask_for_fighstick_data(cs: &CriticalSection) {
  let usart = USART.borrow(cs).borrow();
  let dre = usart.as_ref().unwrap().ucsr1a.read().udre1().bit();
  let intro_complete = INTRO_COMPLETE.borrow(cs).borrow();

  if *intro_complete && dre {
    send_command(&usart, UsartCommand::SendData);
  }
}

#[interrupt(atmega8u2)]
fn USART1_RX() {
  interrupt::free(|cs| {
    let mut sent_intro = SENT_INTRO.borrow(cs).borrow_mut();
    let mut intro_complete = INTRO_COMPLETE.borrow(cs).borrow_mut();
    let mut getting_data = GETTING_DATA.borrow(cs).borrow_mut();

    let usart = USART.borrow(cs).borrow();
    let data = usart.as_ref().unwrap().udr1.read().bits();
    let possible_command: UsartCommand = data.into();

    match possible_command {
      UsartCommand::Introduction => {
        if *sent_intro {
          *sent_intro = false;
          *intro_complete = true;
        }
      },
      UsartCommand::SendData => {
        *getting_data = true;
      },
      UsartCommand::Unknown => {
        if *getting_data {
          let mut table_pointer = FIGHTSTICK_TABLE_POINTER.borrow(cs).borrow_mut();
          let mut staging_table = STAGING_FIGHTSTICK.borrow(cs).borrow_mut();
          let mut table = staging_table.clone();
          table.0[*table_pointer as usize] = data;
          *staging_table = table;
          *table_pointer += 1;

          if *table_pointer >= 4 {
            *table_pointer = 0;
            *getting_data = false;
            FIGHTSTICK.borrow(cs).replace(staging_table.clone());
          }
        }
      },
    }
  });
}
