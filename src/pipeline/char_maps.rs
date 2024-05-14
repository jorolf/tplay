//! This module defines various character maps used for rendering the media.
//!
//! The character maps are strings that represent different sets of characters
//! that can be used to approximate the grayscale levels of the media being rendered.
//!
//! The available character maps are:
//! * `CHARS1`: 10 characters, ASCII-127 only.
//! * `CHARS2`: 67 characters, ASCII-127 only.
//! * `CHARS3`: 92 characters, ASCII-255.
//! * `SOLID`: 1 character, a solid block.
//! * `DOTTED`: 1 character, a dotted block.
//! * `GRADIENT`: 5 characters, a gradient of solid blocks.
//! * `BLACKWHITE`: 2 characters, a solid block and a space.
//! * `BW_DOTTED`: 2 characters, a dotted block and a space.
//! * `BRAILLE`: 16 characters, a braille-based gradient of solid blocks.

use image::{GenericImageView, GrayImage, SubImage};

// ASCII-127 Only
pub const CHARS1: &str = r##" .:-=+*#%@"##; // 10 chars
pub const CHARS2: &str = r##" .'`^",:;Il!i~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"##; // 67 chars
pub const CHARS3: &str = r##" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@"##; // 92 chars

// ASCII-255
pub const SOLID: &str = r#"█"#; // 1 Solid block
pub const DOTTED: &str = r#"⣿"#; // 1 dotted block
pub const GRADIENT: &str = r#" ░▒▓█"#; // 5 chars

pub trait CharMap : Clone {
    fn get_char(&self, image: &SubImage<&GrayImage>) -> char;
    fn get_subpixels(&self) -> (u32, u32);

    fn get_line_prefix(&self) -> &str {
        ""
    }
}

impl CharMap for Vec<char> {
    fn get_char(&self, image: &SubImage<&GrayImage>) -> char {
        let lum = image.get_pixel(0, 0)[0] as u32;
        let lookup_idx = self.len() * lum as usize / (u8::MAX as usize + 1);
        self[lookup_idx]
    }
    
    fn get_subpixels(&self) -> (u32, u32) {
        (1, 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Braille;

impl CharMap for Braille {
    fn get_char(&self, image: &SubImage<&GrayImage>) -> char {
        const BRAILLE_BLANK: u32 = 0x2800;
        let mut braille = BRAILLE_BLANK;

        let braille_dots = [
            ((0, 0), 0x1),
            ((1, 0), 0x8),
            ((0, 1), 0x2),
            ((1, 1), 0x10),
            ((0, 2), 0x4),
            ((1, 2), 0x20),
            ((0, 3), 0x40),
            ((1, 3), 0x80),
        ];

        for ((x, y), braille_bit) in braille_dots {
            if image.get_pixel(x, y)[0] > 127 {
                    braille |= braille_bit;
            }
        }

        char::from_u32(braille).unwrap()
    }

    fn get_subpixels(&self) -> (u32, u32) {
        (2, 4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mosaic;

fn mosaic_char(bitmask: u8) -> char {
    const MOSAIC_BASE: u32 = 0x1FB00;

    match bitmask {
        0 => ' ',
        0x01..=0x14 => char::from_u32(MOSAIC_BASE + bitmask as u32 - 0x01).unwrap(),
        0x15 => '▌',
        0x16..=0x29 => char::from_u32(MOSAIC_BASE + bitmask as u32 - 0x02).unwrap(),
        0x2A => '▐',
        0x2B..=0x3E => char::from_u32(MOSAIC_BASE + bitmask as u32 - 0x03).unwrap(),
        0x3F => '█',
        0x40..=0xFF => panic!("Mosaic bitmask out of bounds: {bitmask:X} > 0x40")
    }
}

impl CharMap for Mosaic {
    fn get_char(&self, image: &SubImage<&GrayImage>) -> char {
        let mut mosaic = 0;

        let mosaic_blocks = [
            ((0, 0), 0x1),
            ((1, 0), 0x2),
            ((0, 1), 0x4),
            ((1, 1), 0x8),
            ((0, 2), 0x10),
            ((1, 2), 0x20),
        ];

        for ((x, y), mosaic_bit) in mosaic_blocks {
            if image.get_pixel(x, y)[0] > 127 {
                    mosaic += mosaic_bit;
            }
        }

        mosaic_char(mosaic)
    }

    fn get_subpixels(&self) -> (u32, u32) {
        (2, 3)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TeletextMosaic;

fn teletext_mosaic_char(bitmask: u8) -> char {
    const MOSAIC_BASE: u32 = 0xE020;
    let bitmask = bitmask as u32;

    if bitmask >= 0x40 {
        panic!("Mosaic bitmask out of bounds: {bitmask:X} > 0x40")
    }

    let mosaic = MOSAIC_BASE + (bitmask & 0x1F) + ((bitmask & 0x20) << 1);

    assert!((0xE000..=0xE0FF).contains(&mosaic));

    char::from_u32(mosaic).unwrap()
}

impl CharMap for TeletextMosaic {
    fn get_char(&self, image: &SubImage<&GrayImage>) -> char {
        let mut mosaic = 0;

        let mosaic_blocks = [
            ((0, 0), 0x1),
            ((1, 0), 0x2),
            ((0, 1), 0x4),
            ((1, 1), 0x8),
            ((0, 2), 0x10),
            ((1, 2), 0x20),
        ];

        for ((x, y), mosaic_bit) in mosaic_blocks {
            if image.get_pixel(x, y)[0] > 127 {
                    mosaic += mosaic_bit;
            }
        }

        teletext_mosaic_char(mosaic)
    }

    fn get_subpixels(&self) -> (u32, u32) {
        (2, 3)
    }

    fn get_line_prefix(&self) -> &str {
        "\u{E017}"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharMaps {
    Simple(Vec<char>),
    Braille,
    Mosaic,
    TeletextMosaic,
}

impl CharMap for CharMaps {
    fn get_char(&self, image: &SubImage<&GrayImage>) -> char {
        match self {
            CharMaps::Simple(vec) => vec.get_char(image),
            CharMaps::Braille => Braille.get_char(image),
            CharMaps::Mosaic => Mosaic.get_char(image),
            CharMaps::TeletextMosaic => TeletextMosaic.get_char(image),
        }
    }

    fn get_subpixels(&self) -> (u32, u32) {
        match self {
            CharMaps::Simple(vec) => vec.get_subpixels(),
            CharMaps::Braille => Braille.get_subpixels(),
            CharMaps::Mosaic => Mosaic.get_subpixels(),
            CharMaps::TeletextMosaic => TeletextMosaic.get_subpixels(),
        }
    }

    fn get_line_prefix(&self) -> &str {
        match self {
            CharMaps::TeletextMosaic => TeletextMosaic.get_line_prefix(),
            _ => ""
        }
    }
}

impl<T> From<T> for CharMaps
    where T: AsRef<str> {
    fn from(value: T) -> Self {
        CharMaps::Simple(value.as_ref().chars().collect())
    }
}
