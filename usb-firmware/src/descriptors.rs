
// OFS
pub const MANUFACTURER: [u8; 8] = [8, 3, 0x4f, 0x00,0x46, 0x00,0x53, 0x00];

// Open Fight Stick v2a
pub const PRODUCT: [u8; 42] = [42, 3, 0x4f, 0x00,0x70, 0x00,0x65, 0x00,0x6e, 0x00,0x20, 0x00,0x46, 0x00,0x69, 0x00,0x67, 0x00,0x68, 0x00,0x74, 0x00,0x20, 0x00,0x53, 0x00,0x74, 0x00,0x69, 0x00,0x63, 0x00,0x6b, 0x00,0x20, 0x00,0x76, 0x00,0x32, 0x00,0x61, 0x00];

pub const VENDOR_ID: u16 = 0x10C4;
pub const PRODUCT_ID: u16 = 0x82C0;

pub const ENDPOINT0_SIZE: u8 = 64;

pub const GAMEPAD_INTERFACE: u8 = 0;
pub const GAMEPAD_ENDPOINT: u8 = 1;
pub const GAMEPAD_SIZE: u8 = 64;
pub const GAMEPAD_BUFFER: u8 = 0x02;

pub const DEVICE_DESCRIPTOR: [u8; 18] = [
  18,
  1,
  0x10,
  0x01,
  0,
  0,
  0,
  ENDPOINT0_SIZE,
  (VENDOR_ID & 0xFF) as u8,
  (VENDOR_ID >> 8) as u8,
  (PRODUCT_ID & 0xFF) as u8,
  (PRODUCT_ID >> 8) as u8,
  0x00,
  0x01,
  1,
  2,
  0,
  1
];

// TODO Fix up Report
pub const HID_REPORT_DESC_SIZE: usize = 55;
pub const HID_REPORT_DESC: [u8; HID_REPORT_DESC_SIZE] = [
	0x05, 0x01,                    // USAGE_PAGE (Generic Desktop)
	0x09, 0x04,                    // USAGE (Gamepad)
	0xa1, 0x01,                    // COLLECTION (Application)
	0xa1, 0x02,                    //   COLLECTION (Logical)
	0x15, 0x00,                    //     LOGICAL_MINIMUM (0)
	0x26, 0xff, 0x00,              //     LOGICAL_MAXIMUM (255)
	0x35, 0x00,                    //     PHYSICAL_MINIMUM (0)
	0x46, 0xff, 0x00,              //     PHYSICAL_MAXIMUM (255)
	0x05, 0x01,                    //     USAGE_PAGE (Generic Desktop)
	0x75, 0x08,                    //     REPORT_SIZE (8)
	0x95, 0x02,                    //     REPORT_COUNT (2)
	0x09, 0x30,                    //     USAGE (X)
	0x09, 0x31,                    //     USAGE (Y)
	0x81, 0x02,                    //     INPUT (Data,Var,Abs)
	0xc0,                          //   END_COLLECTION

	0xa1, 0x02,                    //   COLLECTION (Logical)
	0x05, 0x09,                    //     USAGE_PAGE (Button)
	0x25, 0x01,                    //     LOGICAL_MAXIMUM (1)
	0x15, 0x00,                    //     LOGICAL_MINIMUM (0)
	0x19, 0x01,                    //     USAGE_MINIMUM (Button 1)
	0x29, 0x0B,                    //     USAGE_MAXIMUM (Button 11)
	0x95, 0x0B,                    //     REPORT_COUNT (11)
	0x75, 0x01,                    //     REPORT_SIZE (1)
	0x81, 0x02,                    //     INPUT (Data,Var,Abs)
	0x95, 0x05,                    //     REPORT_COUNT (5)
	0x81, 0x01,                    //     INPUT (Cnst,Ary,Abs)
	0xc0,                          // 	END_COLLECTION
	0xc0                           // END_COLLECTION
];

pub const CONFIG1_DESC_SIZE: usize = 34;
pub const CONFIG1_DESC: [u8; CONFIG1_DESC_SIZE] = [
  9, 					// bLength;
	2,					// bDescriptorType;
  (CONFIG1_DESC_SIZE & 0xFF) as u8,
  (CONFIG1_DESC_SIZE >> 8) as u8,
	1,					// bNumInterfaces
	1,					// bConfigurationValue
	0,					// iConfiguration
	0x80,					// bmAttributes
	50,					// bMaxPower
  // interface descriptor, USB spec 9.6.5, page 267-269, Table 9-12
	9,					// bLength
	4,					// bDescriptorType
	GAMEPAD_INTERFACE,			// bInterfaceNumber
	0,					// bAlternateSetting
	1,					// bNumEndpoints
	0x03,					// bInterfaceClass (0x03 = HID)
	0x00,					// bInterfaceSubClass (0x00 = No Boot)
	0x00,					// bInterfaceProtocol (0x00 = No Protocol)
	0,					// iInterface,
  // HID interface descriptor, HID 1.11 spec, section 6.2.1
	9,					// bLength
	0x21,					// bDescriptorType
	0x11, 0x01,				// bcdHID
	0,					// bCountryCode
	1,					// bNumDescriptors
	0x22,					// bDescriptorType
  (HID_REPORT_DESC_SIZE & 0xFF) as u8,
  0,
	// endpoint descriptor, USB spec 9.6.6, page 269-271, Table 9-13
	7,					// bLength
	5,					// bDescriptorType
	GAMEPAD_ENDPOINT | 0x80,		// bEndpointAddress
	0x03,					// bmAttributes (0x03=intr)
	GAMEPAD_SIZE, 0,			// wMaxPacketSize
	10					// bInterval
];

pub const HID: [u8; 9] = [
	9,					// bLength
	0x21,					// bDescriptorType
	0x11, 0x01,				// bcdHID
	0,					// bCountryCode
	1,					// bNumDescriptors
	0x22,					// bDescriptorType
	(HID_REPORT_DESC_SIZE) as u8,
  0,
];

// TODO Determine what these are and change them if needed
pub static INIT_BYTES: [u8; 8] = [
	0x21, 0x26, 0x01, 0x07, 0x00, 0x00, 0x00, 0x00
];

pub struct Descriptor {
  pub value: u16,
  pub index: u16,
	pub data: &'static [u8]
}

impl Descriptor {
  pub const fn new(value: u16, index: u16, data: &'static [u8]) -> Descriptor {
    Descriptor {
      value,
      index,
      data
    }
  }
}

pub static ENDPOINT_TABLE: [u8; 6] = [
	1,
	0xC1,
	0x30 | GAMEPAD_BUFFER,
	0,
	0,
	0
];

pub static DESCRIPTOR_LIST: [Descriptor; 6] = [
  Descriptor::new(0x0100, 0x0000, &DEVICE_DESCRIPTOR),
  Descriptor::new(0x0200, 0x0000, &CONFIG1_DESC),
  Descriptor::new(0x2200, GAMEPAD_INTERFACE as u16, &HID_REPORT_DESC),
  Descriptor::new(0x0300, 0x0000, &[4, 3, 0x09, 0x04]),
  Descriptor::new(0x0301, 0x0409, &MANUFACTURER),
  Descriptor::new(0x0302, 0x0409, &PRODUCT),
];
