use byteorder::{BigEndian, ByteOrder};
use cairo::Context;
use memmap::{Mmap, MmapOptions};
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64;
use std::{error::Error, fs, io, path::Path, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: i32,
    height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Size {
        Size { width, height }
    }

    pub fn from_arg(s: &str) -> Result<Size, String> {
        match s.parse() {
            Ok(v) => Ok(v),
            Err(_) => Err(format!("invalid size: {}", s)),
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl FromStr for Size {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.find("x") {
            Some(ix) => Ok(Size {
                width: s[..ix].parse()?,
                height: s[ix + 1..].parse()?,
            }),
            None => {
                let size = s.parse()?;
                Ok(Size {
                    width: size,
                    height: size,
                })
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn from_u32(c: u32) -> Color {
        Color {
            r: ((c >> 16) & 0xff) as u8,
            g: ((c >> 8) & 0xff) as u8,
            b: (c & 0xff) as u8,
        }
    }

    pub fn set_with_alpha(&self, ctx: &Context, a: f64) {
        let r = self.r as f64 / 256.0;
        let g = self.g as f64 / 256.0;
        let b = self.b as f64 / 256.0;
        ctx.set_source_rgba(r, g, b, a);
    }

    pub fn set(&self, ctx: &Context) {
        let r = self.r as f64 / 256.0;
        let g = self.g as f64 / 256.0;
        let b = self.b as f64 / 256.0;
        ctx.set_source_rgb(r, g, b);
    }

    pub fn luminance(&self) -> f64 {
        let r = self.r as f64 / 256.0;
        let g = self.g as f64 / 256.0;
        let b = self.b as f64 / 256.0;
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

const THEME_SIZE: usize = 20;

#[derive(Debug)]
pub struct Themes {
    mem: Mmap,
}

impl Themes {
    pub fn open<P: AsRef<Path>>(src: P) -> io::Result<Self> {
        let f = fs::File::open(src)?;
        Ok(Themes {
            mem: unsafe { MmapOptions::new().map(&f)? },
        })
    }

    pub fn get(&self, idx: usize) -> Vec<Color> {
        let off = idx * THEME_SIZE;
        let mut colors = Vec::with_capacity(5);
        for i in 0..5 {
            let b = off + i * 4;
            colors.push(Color::from_u32(BigEndian::read_u32(&self.mem[b..b + 4])));
        }
        colors
    }

    pub fn pick(&self, rng: &mut dyn rand::RngCore) -> (usize, Vec<Color>) {
        let ix = Uniform::new(0, self.len()).sample(rng);
        (ix, self.get(ix))
    }

    pub fn len(&self) -> usize {
        self.mem.len() / THEME_SIZE
    }
}

pub trait RenderOpts {
    fn size(&self) -> Size;

    fn rng(&self) -> Pcg64;

    fn themes(&self) -> io::Result<Themes>;
}
