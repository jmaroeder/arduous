use std::error::Error;

pub const DISPLAY_WIDTH: usize = 128;
pub const DISPLAY_HEIGHT: usize = 64;
pub const DISPLAY_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

#[derive(Default)]
struct ButtonState {
    a: bool,
    b: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

pub struct Arduboy {
    button_state: ButtonState,
    display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Default for Arduboy {
    fn default() -> Arduboy {
        Arduboy {
            button_state: Default::default(),
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }
}

impl Arduboy {
    pub fn display(&self) -> &[[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT] {
        return &self.display;
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        // TODO
        Ok(())
    }

    pub fn execute_for_a_frame(&self) -> Result<(), Box<dyn Error>> {
        // TODO
        self.execute()?;
        Ok(())
    }

    pub fn set_button_state(&mut self, button: Button, state: bool) {
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

pub enum Button {
    A,
    B,
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_display_inits() {
        let arduboy = Arduboy {
            ..Default::default()
        };
        for row in arduboy.display().iter() {
            for pixel in row.iter() {
                assert_eq!(pixel, &false);
            }
        }
    }
}
