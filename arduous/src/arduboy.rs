use crate::ssd1306::SSD1306;
use std::error::Error;

pub enum Button {
  A,
  B,
  Up,
  Down,
  Left,
  Right,
}

#[derive(Default)]
struct ButtonState {
  a: bool,
  b: bool,
  up: bool,
  down: bool,
  left: bool,
  right: bool,
}

#[derive(Default)]
pub struct Arduboy {
  button_state: ButtonState,
  display: SSD1306,

  test_inited: bool,
}

const TEST_COMMANDS: [u8; 2] = [0x20u8, 0x00];
const TEST_DATA: [u8; 8] = [0x01, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x7F, 0xFF];

impl Arduboy {
  pub fn display_ssd1306(&self) -> &SSD1306 {
    return &self.display;
  }

  pub fn display_iter(&self) -> impl Iterator<Item = bool> + '_ {
    return self.display.iter();
  }

  pub fn execute(&mut self) -> Result<(), Box<dyn Error>> {
    // TODO
    if !self.test_inited {
      for command in TEST_COMMANDS.iter() {
        self.display.push_command(*command);
      }
      self.test_inited = true;
    }
    for data in TEST_DATA.iter() {
      self.display.push_data(*data);
    }

    Ok(())
  }

  pub fn execute_for_a_frame(&mut self) -> Result<(), Box<dyn Error>> {
    // TODO
    self.execute()?;

    Ok(())
  }

  pub fn set_button(&mut self, button: Button, state: bool) {
    let button_state_member = match button {
      Button::A => &mut self.button_state.a,
      Button::B => &mut self.button_state.b,
      Button::Up => &mut self.button_state.up,
      Button::Down => &mut self.button_state.down,
      Button::Left => &mut self.button_state.left,
      Button::Right => &mut self.button_state.right,
    };
    *button_state_member = state;
  }
}

#[cfg(test)]
mod tests {
  // use super::*;
}
