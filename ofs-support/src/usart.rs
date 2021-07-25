pub enum UsartCommand {
  Introduction,
  SendData,
  Unknown,
}

pub const INTRODUCTION: u8 = 0x30;
pub const SEND_DATA: u8 = 0x31;
pub const UNKNOWN: u8 = 0x00;

impl Into<u8> for UsartCommand {
  fn into(self) -> u8 {
    match self {
      Self::Introduction => INTRODUCTION,
      Self::SendData => SEND_DATA,
      Self::Unknown => UNKNOWN,
    }
  }
}

impl From<u8> for UsartCommand {
  fn from(data: u8) -> Self {
    match data {
      INTRODUCTION => Self::Introduction,
      SEND_DATA => Self::SendData,
      _ => Self::Unknown,
    }
  }
}
