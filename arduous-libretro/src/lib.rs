use arduous::{Arduboy, Button};
use libretro_backend::{
    libretro_core, AudioVideoInfo, CoreInfo, GameData, JoypadButton, LoadGameResult, PixelFormat,
    Region, RuntimeHandle,
};
use std::mem;
use std::slice;

const WHITE: u16 = 0xFFFF;
const BLACK: u16 = 0x0000;

struct Emulator {
    arduboy: Arduboy,
    audio_buffer: Vec<i16>,
    framebuffer: [u16; 128 * 64],
    game_data: Option<GameData>,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            arduboy: Arduboy::default(),
            audio_buffer: Vec::with_capacity(44100 * 2),
            framebuffer: [0; 128 * 64],
            game_data: None,
        }
    }
}

impl libretro_backend::Core for Emulator {
    fn info() -> CoreInfo {
        CoreInfo::new("arduous", env!("CARGO_PKG_VERSION"))
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        if game_data.is_empty() {
            return LoadGameResult::Failed(game_data);
        }
        self.game_data = Some(game_data);
        LoadGameResult::Success(
            AudioVideoInfo::new()
                .video(128, 64, 60.0, PixelFormat::RGB565)
                .audio(44100.0)
                .region(Region::NTSC),
        )
    }

    fn on_unload_game(&mut self) -> GameData {
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        macro_rules! update_controllers {
            ( $( $button:ident ),+ ) => (
                $(
                    self.arduboy.set_button(Button::$button, handle.is_joypad_button_pressed( 0, JoypadButton::$button ) );
                )+
            )
        }

        update_controllers!(A, B, Up, Down, Left, Right);

        self.arduboy.execute_for_a_frame();

        self.update_framebuffer();
        handle.upload_video_frame(as_bytes(&self.framebuffer[..]));

        self.finish_audio_buffer();
        handle.upload_audio_frame(&self.audio_buffer[..]);
        self.audio_buffer.clear();
    }

    fn on_reset(&mut self) {}
}

impl Emulator {
    fn update_framebuffer(&mut self) {
        for (pixel_in, pixel_out) in self.arduboy.display_iter().zip(self.framebuffer.iter_mut()) {
            *pixel_out = Emulator::translate_pixel(pixel_in);
        }
    }

    fn finish_audio_buffer(&mut self) {
        // hack to add silence
        self.audio_buffer.resize(44100 / 60 * 2, 0);
    }

    fn translate_pixel(value: bool) -> u16 {
        match value {
            false => BLACK,
            true => WHITE,
        }
    }
}

fn as_bytes<T: Copy>(array: &[T]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            mem::transmute(array.as_ptr()),
            mem::size_of::<T>() * array.len(),
        )
    }
}

libretro_core!(Emulator);
