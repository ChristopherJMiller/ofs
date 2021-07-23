
pub struct Fightstick {
  pub iter_count: u8,

  pub x: i8,
  pub y: i8,

  pub button_0: bool,
  pub button_1: bool,
  pub button_2: bool,
  pub button_3: bool,
  pub button_4: bool,
  pub button_5: bool,
  pub button_6: bool,
  pub button_7: bool,
  pub button_8: bool,
  pub button_9: bool,
  pub button_10: bool,
}

impl Default for Fightstick {
  fn default() -> Self {
    Self {
      iter_count: 0,
      x: 0,
      y: 0,
      button_0: false,
      button_1: false,
      button_2: false,
      button_3: false,
      button_4: false,
      button_5: false,
      button_6: false,
      button_7: false,
      button_8: false,
      button_9: false,
      button_10: false,
    }
  }
}

#[inline(always)]
fn left_shift_bit(val: bool, index: u8) -> u8 {
  if val {
    1 << index
  } else {
    0 << index
  }
}

impl Fightstick {
  pub fn get_descriptor_index(&self, index: u8) -> Option<u8> {
    match index {
      0 => Some((self.x + 127) as u8),
      1 => Some((self.y + 127) as u8),
      2 => {
        Some(
          left_shift_bit(self.button_0, 0)
          | left_shift_bit(self.button_1, 1)
          | left_shift_bit(self.button_2, 2)
          | left_shift_bit(self.button_3, 3)
          | left_shift_bit(self.button_4, 4)
          | left_shift_bit(self.button_5, 5)
          | left_shift_bit(self.button_6, 6)
          | left_shift_bit(self.button_7, 7)
        )
      },
      3 => {
        Some(
          left_shift_bit(self.button_8, 0)
          | left_shift_bit(self.button_9, 1)
          | left_shift_bit(self.button_10, 2)
        )
      },
      _ => None
    }
  }
}
