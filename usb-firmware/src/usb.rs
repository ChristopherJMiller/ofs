
use avr_device::atmega8u2::{PLL, USB_DEVICE, PORTD};
use avr_device::interrupt::{free, Mutex, CriticalSection};
use core::cell::{Ref, RefCell};
use avr_device::interrupt;
use ofs_support::fightstick::Fightstick;

use crate::descriptors::{DESCRIPTOR_LIST, ENDPOINT0_SIZE, ENDPOINT_TABLE, GAMEPAD_ENDPOINT, GAMEPAD_INTERFACE, INIT_BYTES};

pub static PORTD: Mutex<RefCell<Option<PORTD>>> = Mutex::new(RefCell::new(None));
pub static USB_DEVICE: Mutex<RefCell<Option<USB_DEVICE>>> = Mutex::new(RefCell::new(None));
pub static USB_CONFIGURED: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));
pub static USB_IDLE_CONFIG: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));
pub static USB_PROTOCOL: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(1));

pub enum RequestType {
  GetStatus,
  ClearFeature,
  SetFeature,
  SetAddress,
  GetDescriptor,
  GetConfiguration,
  SetConfiguration,
  GetInterface,
  SetInterface,
  HidGetReport,
  HidSetReport,
  HidGetIdle,
  HidSetIdle,
  HidGetProtocol,
  HidSetProtocol,
  Stall,
}

impl RequestType {
  pub fn from_u8(request_type: u8, request_num: u8, index: u8) -> RequestType {
    match (request_type, request_num, index) {
      (0x80, 8, _) => RequestType::GetConfiguration,
      (0, 9, _) => RequestType::SetConfiguration,
      (0xA1, 1, GAMEPAD_INTERFACE) => RequestType::HidGetProtocol,
      (0xA1, 2, GAMEPAD_INTERFACE) => RequestType::HidGetIdle,
      (0xA1, 3, GAMEPAD_INTERFACE) => RequestType::HidGetProtocol,
      (0x21, 9, GAMEPAD_INTERFACE) => RequestType::HidSetReport,
      (0x21, 10, GAMEPAD_INTERFACE) => RequestType::HidSetIdle,
      (0x21, 11, GAMEPAD_INTERFACE) => RequestType::HidSetProtocol,
      (_, 0, _) => RequestType::GetStatus,
      (_, 5, _) => RequestType::SetAddress,
      (_, 6, _) => RequestType::GetDescriptor,
      _ => RequestType::Stall,
    }
  }
}

pub fn setup_usb(cs: &CriticalSection, usb: USB_DEVICE, pll: PLL, portd: PORTD) {
  usb.usbcon.write(|w| w
    .frzclk().set_bit()
    .usbe().set_bit()
  );

  pll.pllcsr.write(|w| unsafe { w.bits(1 << 2).plle().set_bit() });

  while pll.pllcsr.read().plock().bit_is_clear() {}

  usb.usbcon.write(|w| w.usbe().set_bit());
  usb.udcon.write(|w| unsafe { w.bits(0) });
  usb.udien.write(|w| w.eorste().set_bit().sofe().set_bit());
  
  portd.ddrd.write(|w| w.pd5().set_bit().pd4().set_bit());
  portd.portd.write(|w| w.pd5().set_bit().pd4().set_bit());

  USB_DEVICE.borrow(cs).replace(Some(usb));
  PORTD.borrow(cs).replace(Some(portd));
}

#[interrupt(atmega8u2)]
fn USB_GEN() {
  free(|cs| {
    let usb = USB_DEVICE.borrow(cs).borrow();

    let eorsti = usb.as_ref().unwrap().udint.read().eorsti().bit();
    usb.as_ref().unwrap().udint.write(|w| unsafe { w.bits(0) });

    if eorsti {
      usb.as_ref().unwrap().uenum.write(|w| unsafe { w.bits(0) });
      usb.as_ref().unwrap().ueconx.write(|w| unsafe { w.bits(1) });
      usb.as_ref().unwrap().uecfg0x.write(|w| w.eptype().bits(0));
      usb.as_ref().unwrap().uecfg1x.write(|w| w
        .epsize().bits(0x3) // 64 Bytes
        .alloc().set_bit()
      );
      usb.as_ref().unwrap().ueienx.write(|w| w.rxstpe().set_bit());
      *USB_CONFIGURED.borrow(cs).borrow_mut() = 0;
    }
  });
}

fn usb_send_in(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>) {
  usb.as_ref().unwrap().ueintx.modify(|_, w| w.txini().clear_bit());
}

fn usb_wait_in_ready(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>) {
  loop {
    let txini = usb.as_ref().unwrap().ueintx.read().txini().bit();
    if txini {
      break;
    }
  }
}

fn wait_for_host_ready(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>) {
  loop {
    let txini = usb.as_ref().unwrap().ueintx.read().txini().bit();
    let rxouti = usb.as_ref().unwrap().ueintx.read().rxouti().bit();
    if !txini || !rxouti {
      return;
    }
  }
}

fn usb_wait_receive_out(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>) {
  loop {
    let rxouti = usb.as_ref().unwrap().ueintx.read().rxouti().bit();
    if !rxouti {
      return;
    }
  }
}

fn usb_ack_out(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>) {
  usb.as_ref().unwrap().ueintx.write(|w| unsafe { w.bits(u8::max_value()).rxouti().clear_bit() });
}

pub fn send_gamepad_data(cs: &CriticalSection) {
  let usb = USB_DEVICE.borrow(cs).borrow();
  let config = USB_CONFIGURED.borrow(cs).borrow();

  PORTD.borrow(cs).borrow().as_ref().unwrap().portd.modify(|r, w| w.pd5().bit(!r.pd5().bit()));

  if *config == 0 {
    return;
  }

  usb.as_ref().unwrap().uenum.write(|w| unsafe { w.bits(GAMEPAD_ENDPOINT) });
  let timeout: u16 = usb.as_ref().unwrap().udfnum.read().bits() + 50;

  loop {
    let rwal = usb.as_ref().unwrap().ueintx.read().rwal().bit();
    if rwal {
      break;
    }

    if *config == 0 {
      return;
    }

    let udfnum = usb.as_ref().unwrap().udfnum.read().bits();
    if udfnum >= timeout {
      return;
    }
  }

  let fightstick = Fightstick {
    x: -100,
    button_0: true,
    button_1: true,
    button_2: true,
    button_3: false,
    button_4: false,
    button_5: true,
    button_6: true,
    button_7: true,
    button_8: true,
    button_9: false,
    button_10: true,
    ..Default::default()
  };

  for i in 0..=3 {
    usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(fightstick.get_descriptor_index(i).unwrap()) });
  }
  
  usb.as_ref().unwrap().ueintx.write(|w| unsafe { w.bits(0x3A) });
}

fn get_descriptor(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>, value: u16, index: u16, length: u16) {
  let descriptor_option = DESCRIPTOR_LIST.iter().find(|f| f.value == value && f.index == index);
  if let Some(descriptor) = descriptor_option {
    let mut len = (length.min(255) as u8).min(descriptor.data.len() as u8);
    let mut table_index: u8 = 0;
    loop {
      wait_for_host_ready(cs, usb);
      if usb.as_ref().unwrap().ueintx.read().rxouti().bit() {
        return;
      }
      let n = ENDPOINT0_SIZE.min(len);
      for _ in 0..n {
        let data = descriptor.data[table_index as usize];
        usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(data) });
        table_index += 1;
      }
      len -= n;
      usb_send_in(cs, usb);

      if !(len > 0 || n == ENDPOINT0_SIZE) {
        break;
      }
    } 
    return;
  }
  stall(cs, usb);
}

fn set_address(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>, value: u16) {
  usb_send_in(cs, usb);
  usb_wait_in_ready(cs, usb);
  usb.as_ref().unwrap().udaddr.write(|w| w.uadd().bits(value as u8).adden().set_bit());
}

fn set_configuration(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>, value: u16) {
  *USB_CONFIGURED.borrow(cs).borrow_mut() = value as u8;
  usb_send_in(cs, usb);
  let mut table_index = 0;
  for i in 1..5 {
    let en = ENDPOINT_TABLE[table_index];
    usb.as_ref().unwrap().uenum.write(|w| unsafe { w.bits(i as u8) });
    usb.as_ref().unwrap().ueconx.write(|w| unsafe { w.bits(en) });
    table_index += 1;
    if en > 0 {
      usb.as_ref().unwrap().uecfg0x.write(|w| unsafe { w.bits(ENDPOINT_TABLE[table_index]) });
      table_index += 1;
      usb.as_ref().unwrap().uecfg1x.write(|w| unsafe { w.bits(ENDPOINT_TABLE[table_index]) });
      table_index += 1;
    }
  }
  usb.as_ref().unwrap().uerst.write(|w| unsafe { w.bits(0x1E) });
  usb.as_ref().unwrap().uerst.write(|w| unsafe { w.bits(0) });
}

fn stall(cs: &CriticalSection, usb: &Ref<Option<USB_DEVICE>>) {
  usb.as_ref().unwrap().ueconx.write(|w| w.stallrq().set_bit().epen().set_bit());
}

#[interrupt(atmega8u2)]
fn USB_COM() {
  free(|cs| {
    let usb = USB_DEVICE.borrow(cs).borrow();

    usb.as_ref().unwrap().uenum.write(|w| unsafe { w.bits(0) });
    let rxstpi = usb.as_ref().unwrap().ueintx.read().rxstpi().bit();

    if rxstpi {
      let request_type = usb.as_ref().unwrap().uedatx.read().bits();
      let request = usb.as_ref().unwrap().uedatx.read().bits();

      let mut value = usb.as_ref().unwrap().uedatx.read().bits() as u16;
      value |= (usb.as_ref().unwrap().uedatx.read().bits() as u16) << 8;

      let mut index = usb.as_ref().unwrap().uedatx.read().bits() as u16;
      index |= (usb.as_ref().unwrap().uedatx.read().bits() as u16) << 8;

      let mut length = usb.as_ref().unwrap().uedatx.read().bits() as u16;
      length |= (usb.as_ref().unwrap().uedatx.read().bits() as u16) << 8;

      usb.as_ref().unwrap().ueintx.write(|w| unsafe { 
        w
        .bits(u8::max_value()) 
        .rxstpi().clear_bit()
        .rxouti().clear_bit()
        .txini().clear_bit()
      });

      PORTD.borrow(cs).borrow().as_ref().unwrap().portd.modify(|r, w| w.pd4().bit(!r.pd4().bit()));

      match RequestType::from_u8(request_type, request, index as u8) {
        RequestType::GetDescriptor => {
          get_descriptor(cs, &usb, value, index, length);
        },
        RequestType::SetAddress => set_address(cs, &usb, value),
        RequestType::SetConfiguration => {
          if request_type == 0 {
            set_configuration(cs, &usb, value);
          } else {
            stall(cs, &usb);
          }
        },
        RequestType::GetConfiguration => {
          if request_type == 0x80 {
            usb_wait_in_ready(cs, &usb);
            let config = USB_CONFIGURED.borrow(cs).borrow();
            usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(*config)});
            usb_send_in(cs, &usb);
          } else {
            stall(cs, &usb)
          }
        },
        RequestType::GetStatus => {
          usb_wait_in_ready(cs, &usb);
          usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(0) });
          usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(0) });
          usb_send_in(cs, &usb);
        },
        RequestType::HidGetIdle => {
          let idle = USB_IDLE_CONFIG.borrow(cs).borrow();
          usb_wait_in_ready(cs, &usb);
          usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(*idle) });
          usb_send_in(cs, &usb);
        },
        RequestType::HidGetProtocol => {
          let protocol = USB_IDLE_CONFIG.borrow(cs).borrow();
          usb_wait_in_ready(cs, &usb);
          usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(*protocol) });
          usb_send_in(cs, &usb);
        },
        RequestType::HidGetReport => {
          usb_wait_in_ready(cs, &usb);
          for data in INIT_BYTES.iter() {
            usb.as_ref().unwrap().uedatx.write(|w| unsafe { w.bits(*data) });
          }

          usb_send_in(cs, &usb);
        },
        RequestType::HidSetReport => {
          usb_wait_receive_out(cs, &usb);
          usb_ack_out(cs, &usb);
          usb_send_in(cs, &usb);
        },
        RequestType::HidSetIdle => {
          *USB_IDLE_CONFIG.borrow(cs).borrow_mut() = (value >> 8) as u8;
          usb_send_in(cs, &usb);
        },
        RequestType::HidSetProtocol => {
          *USB_PROTOCOL.borrow(cs).borrow_mut() = value as u8;
          usb_send_in(cs, &usb);
        },
        RequestType::Stall => stall(cs, &usb),
        _ => stall(cs, &usb)
      }
    }
  });
}
