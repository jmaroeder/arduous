use bitvec::prelude::*;

pub const DISPLAY_WIDTH: usize = 128;
pub const DISPLAY_HEIGHT: usize = 64;
pub const DISPLAY_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

const PAGE_SIZE: usize = 8;

#[repr(u8)]
pub enum AddressingMode {
    Horizontal = 0,
    Vertical = 1,
    Page = 2,
}

pub struct SSD1306 {
    // registers
    alternative_com: bool,
    com_remap: bool,
    ignore_ram: bool,
    inverted: bool,
    segment_remap: bool,
    sleeping: bool,

    addressing_mode: AddressingMode,

    col: u8,
    column_end: u8,
    column_start: u8,
    contrast: u8,
    divide_ratio: u8,
    display_start_line: u8,
    multiplex_ratio: u8,
    oscillator_frequency: u8,
    page_end: u8,
    page_mode_column_start: u8,
    page_mode_page_start: u8,
    page_start: u8,
    page: u8,
    precharge_period: u8,
    vcomh_deselect_level: u8,
    vertical_shift: u8,

    // other state
    command_buffer: Vec<BitArr!(for 8, in Lsb0, u8)>,
    vram: BitArr!(for DISPLAY_PIXELS, in usize),
}

impl Default for SSD1306 {
    fn default() -> SSD1306 {
        SSD1306 {
            alternative_com: true,
            com_remap: false,
            ignore_ram: false,
            inverted: false,
            segment_remap: false,
            sleeping: false,

            addressing_mode: AddressingMode::Page,

            col: 0,
            column_end: 0x7f,
            column_start: 0,
            contrast: 0x7f,
            display_start_line: 0,
            divide_ratio: 1,
            multiplex_ratio: 64,
            oscillator_frequency: 0x08,
            page_end: 0x07,
            page_mode_column_start: 0,
            page_mode_page_start: 0,
            page_start: 0,
            page: 0,
            precharge_period: 0x22,
            vcomh_deselect_level: 0x20,
            vertical_shift: 0,

            command_buffer: Default::default(),
            vram: bitarr![0; DISPLAY_PIXELS],
        }
    }
}

impl SSD1306 {
    pub fn iter(&self) -> impl Iterator<Item = bool> + '_ {
        let mut filter: fn(bool) -> bool = |b| b;
        if self.sleeping {
            filter = |_| false;
        } else if self.ignore_ram {
            filter = |_| true;
        } else if self.inverted {
            filter = |b| !b;
        }
        self.vram.iter().by_val().map(move |b| filter(b))
    }

    pub fn push_data(&mut self, data: u8) {
        // let x = self.col;
        let data = BitSlice::<Lsb0, _>::from_element(&data);
        let y = ((self.page + 1) as usize * PAGE_SIZE) - 1;
        for i in 0..8 {
            self.set_pixel(self.col as usize, (y - i) as usize, data[i]);
        }

        match self.addressing_mode {
            AddressingMode::Page => {
                self.col += 1;
                if self.col as usize >= DISPLAY_WIDTH {
                    self.col = self.page_mode_column_start;
                }
            }
            AddressingMode::Horizontal => {
                self.col += 1;
                if self.col > self.column_end {
                    self.col = self.column_start;
                    self.page += 1;
                    if self.page > self.page_end {
                        self.page = self.page_start;
                    }
                }
            }
            AddressingMode::Vertical => {
                self.page += 1;
                if self.page > self.page_end {
                    self.page = self.page_start;
                    self.col += 1;
                    if self.col > self.column_end {
                        self.col = self.column_start;
                    }
                }
            }
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        self.vram.set(y * DISPLAY_WIDTH + x, value);
    }

    pub fn push_command(&mut self, data: u8) {
        self.command_buffer.push(BitArray::new([data; 1]));
        if self.command_buffer.len() == self.command_length() {
            self.dispatch_command();
            self.command_buffer.clear();
        }
    }

    fn dispatch_command(&mut self) {
        let command = self.command_buffer[0][0..8].load::<u8>();
        match command {
            0x81 => self.set_contrast_control(),
            0xa4 | 0xa5 => self.entire_display_on(),
            0xa6 | 0xa7 => self.set_normal_inverse_display(),
            0xae | 0xaf => self.set_display_on_off(),
            0x26 | 0x27 | 0x29 | 0x2a | 0x2e | 0x2f | 0xa3 => {
                eprintln!("Unhandled Scrolling Command: {}", command)
            }
            0x00..=0x0f => self.set_lower_column_start_address_for_paging_address_mode(),
            0x10..=0x1f => self.set_higher_column_start_address_for_paging_address_mode(),
            0x20 => self.set_memory_addressing_mode(),
            0x21 => self.set_column_address(),
            0x22 => self.set_page_address(),
            0xb0..=0xb7 => self.set_page_start_address_for_page_addressing_mode(),
            0x40..=0x7f => self.set_display_start_line(),
            0xa0 | 0xa1 => self.set_segment_remap(),
            0xa8 => self.set_multiplex_ratio(),
            0xc0 | 0xc8 => self.set_com_output_scan_direction(),
            0xd3 => self.set_display_offset(),
            0xda => self.set_com_pins_hardware_configuration(),
            0xd5 => self.set_display_clock_divide_ratio(),
            0xd9 => self.set_precharge_period(),
            0xdb => self.set_vcomh_deselect_level(),
            0xe3 => self.nop(),
            _ => panic!("Unknown display command {}", command),
        }
    }

    fn set_contrast_control(&mut self) {
        self.contrast = self.command_buffer[1].as_raw_slice()[0];
    }

    fn entire_display_on(&mut self) {
        self.ignore_ram = self.command_buffer[0][0];
    }

    fn set_normal_inverse_display(&mut self) {
        self.inverted = self.command_buffer[0][0];
    }

    fn set_display_on_off(&mut self) {
        self.sleeping = !self.command_buffer[0][0];
    }

    fn set_lower_column_start_address_for_paging_address_mode(&mut self) {
        self.page_mode_column_start &= 0xF0;
        self.page_mode_column_start |= self.command_buffer[0][0..4].load::<u8>();
        self.col = self.page_mode_column_start;
    }

    fn set_higher_column_start_address_for_paging_address_mode(&mut self) {
        self.page_mode_column_start &= 0x0F;
        self.page_mode_column_start |= self.command_buffer[0][0..4].load::<u8>() << 4;
    }

    fn set_memory_addressing_mode(&mut self) {
        self.addressing_mode = match self.command_buffer[1][0..2].load::<u8>() {
            0b00 => AddressingMode::Horizontal,
            0b01 => AddressingMode::Vertical,
            0b10 => AddressingMode::Page,
            _ => panic!("Unknown addressing mode: {}", self.command_buffer[1]),
        };
    }

    fn set_column_address(&mut self) {
        self.column_start = self.command_buffer[1][0..7].load::<u8>();
        self.column_end = self.command_buffer[2][0..7].load::<u8>();
        self.col = self.column_start;
    }

    fn set_page_address(&mut self) {
        self.page_start = self.command_buffer[1][0..3].load::<u8>();
        self.page_end = self.command_buffer[2][0..3].load::<u8>();
        self.page = self.page_start;
    }

    fn set_page_start_address_for_page_addressing_mode(&mut self) {
        self.page_mode_page_start = self.command_buffer[0][0..3].load::<u8>();
        self.page = self.page_mode_page_start;
    }

    fn set_display_start_line(&mut self) {
        self.display_start_line = self.command_buffer[0][0..6].load::<u8>();
    }

    fn set_segment_remap(&mut self) {
        self.segment_remap = self.command_buffer[0][0];
    }

    fn set_multiplex_ratio(&mut self) {
        let a = self.command_buffer[1][0..6].load::<u8>();
        self.multiplex_ratio = match a {
            15..=63 => a + 1,
            _ => panic!("Invalid multiplex ratio: {}", a),
        }
    }

    fn set_com_output_scan_direction(&mut self) {
        self.com_remap = self.command_buffer[0][3];
    }

    fn set_display_offset(&mut self) {
        self.vertical_shift = self.command_buffer[1][0..6].load::<u8>();
    }

    fn set_com_pins_hardware_configuration(&mut self) {
        self.alternative_com = self.command_buffer[1][4];
        self.com_remap = self.command_buffer[1][5];
    }

    fn set_display_clock_divide_ratio(&mut self) {
        self.divide_ratio = self.command_buffer[1][0..4].load::<u8>() + 1;
        self.oscillator_frequency = self.command_buffer[1][4..8].load::<u8>();
    }

    fn set_precharge_period(&mut self) {
        if self.command_buffer[1][0..4].load::<u8>() == 0
            || self.command_buffer[1][4..8].load::<u8>() == 0
        {
            panic!("Invalid pre-charge period: {}", self.command_buffer[1]);
        }
        self.precharge_period = self.command_buffer[1].load::<u8>();
    }

    fn set_vcomh_deselect_level(&mut self) {
        self.vcomh_deselect_level = self.command_buffer[1][4..7].load::<u8>();
    }

    fn nop(&self) {}

    fn command_length(&self) -> usize {
        match self.command_buffer[0][0..8].load::<u8>() {
            0x81 => 2,        // Set Contrast Control
            0x26 | 0x27 => 7, // Continuous Horizontal Scroll Setup
            0x29 | 0x2A => 6, // Continuous Vertical and Horizontal Scroll Setup
            0xa3 => 3,        // Set Vertical Scroll Area
            0x20 => 2,        // Set Memory Addressing Mode
            0x21 => 3,        // Set Column Address
            0x22 => 3,        // Set Page Address
            0xa8 => 2,        // Set Multiple Ratio
            0xd3 => 2,        // Set Display Offset
            0xda => 2,        // Set COM Pins Hardware Configuration
            0xd5 => 2,        // Set Display Clock Divide Ratio/Oscillator Frequency
            0xd9 => 2,        // Set Pre-charge Period
            0xdb => 2,        // Set V_COMH Deselect Level
            _ => 1,           // All other commands are single-byte
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_inits() {
        let display = SSD1306::default();
        for pixel in display.iter() {
            assert_eq!(pixel, false);
        }
    }
}
