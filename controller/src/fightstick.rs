
use core::cell::RefCell;

use avr_device::{asm::nop, atmega328p::PORTD, interrupt::{CriticalSection, Mutex}};
use ofs_support::fightstick::Fightstick;

static G_PORTD: Mutex<RefCell<Option<PORTD>>> = Mutex::new(RefCell::new(None));

pub fn setup_ports(cs: &CriticalSection, portd: PORTD) {
  portd.ddrd.modify(|_, w| 
    w
      .pd2().set_bit()
      .pd3().set_bit()
      .pd4().clear_bit()
      .pd5().clear_bit()
      .pd6().clear_bit()
      .pd7().clear_bit()
  );

  portd.portd.modify(|_r, w|
    w
      .pd2().clear_bit()
      .pd3().clear_bit()
      .pd4().set_bit()
      .pd5().set_bit()
      .pd6().set_bit()
      .pd7().set_bit()
  );

  G_PORTD.borrow(cs).replace(Some(portd));
}

fn get_line_group(portd: &PORTD, group: u8) -> [bool; 4] {
  let bit0 = (group & 1) > 0;
  let bit1 = (group & 2) > 0;

  portd.portd.write(|w| w.pd2().bit(bit0).pd3().bit(bit1));

  nop();

  [
    portd.pind.read().pd4().bit(),
    portd.pind.read().pd5().bit(),
    portd.pind.read().pd6().bit(),
    portd.pind.read().pd7().bit(),
  ]
}

fn determine_axis(pos: bool, neg: bool) -> i8 {
  let go_pos = pos && !neg;
  let go_neg = neg && !pos;

  if go_pos {
    127
  } else if go_neg {
    -127
  } else {
    0
  }
}

pub fn build_fightstick_data(cs: &CriticalSection) -> Fightstick {
  let portd = G_PORTD.borrow(cs).borrow();

  if let Some(portd) = portd.as_ref() {
    let group_0 = get_line_group(&portd, 0);
    let group_1 = get_line_group(&portd, 1);
    let group_2 = get_line_group(&portd, 2);
    let group_3 = get_line_group(&portd, 3);

    let joystick_up = group_0[2];
    let joystick_right = group_3[2];
    let joystick_down = group_1[2];
    let joystick_left = group_2[2];

    let u_a = !group_0[1];
    let u_b = !group_2[1];
    let u_c = !group_3[0];
    let u_d = !group_1[0];

    let d_a = !group_1[1];
    let d_b = !group_3[1];
    let d_c = !group_2[0];
    let d_d = !group_0[0];

    let start = !group_0[3];

    let x = determine_axis(joystick_left, joystick_right);
    let y = determine_axis(joystick_up, joystick_down);

    Fightstick {
      x,
      y,
      button_0: u_a,
      button_1: u_b,
      button_9: d_a,
      button_3: d_b,
      
      button_4: u_c,
      button_2: u_d,
      button_6: d_c,
      button_7: d_d,

      button_8: start,
      ..Default::default()
    }
  } else {
    Fightstick {
      button_1: true,
      ..Default::default()
    }
  }
}
