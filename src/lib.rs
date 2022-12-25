use byteorder::{BigEndian, ByteOrder};
use cairo::Context;
use core::fmt;
use memmap::{Mmap, MmapOptions};
use rand::{distributions::Uniform, prelude::Distribution};
use rand_pcg::Pcg64;
use std::{error::Error, fs, io, path::Path, str::FromStr};

mod a;
mod b;
mod c;

pub mod common;

const DARKER: f64 = 0.7;
const BRIGHTER: f64 = 1.0 / DARKER;

#[derive(Debug, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:0.3}, {:0.3})", self.x, self.y)
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    tl: Point,
    br: Point,
}

impl Rect {
    pub fn from_wh(w: f64, h: f64) -> Self {
        Self::from_ltrb(0.0, 0.0, w, h)
    }

    pub fn from_xywh(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect {
            tl: Point::new(x, y),
            br: Point::new(x + w, y + h),
        }
    }

    pub fn from_ltrb(l: f64, t: f64, r: f64, b: f64) -> Self {
        Self {
            tl: Point::new(l, t),
            br: Point { x: r, y: b },
        }
    }

    pub fn new(tl: &Point, br: &Point) -> Self {
        Self {
            tl: tl.clone(),
            br: br.clone(),
        }
    }

    pub fn x(&self) -> f64 {
        self.tl.x
    }

    pub fn y(&self) -> f64 {
        self.tl.y
    }

    pub fn width(&self) -> f64 {
        self.br.x - self.tl.x
    }

    pub fn height(&self) -> f64 {
        self.br.y - self.tl.y
    }

    pub fn left(&self) -> f64 {
        self.tl.x
    }

    pub fn right(&self) -> f64 {
        self.br.x
    }

    pub fn top(&self) -> f64 {
        self.tl.y
    }

    pub fn bottom(&self) -> f64 {
        self.br.y
    }

    pub fn top_left(&self) -> &Point {
        &self.tl
    }

    pub fn bottom_right(&self) -> &Point {
        &self.br
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}]", self.tl, self.br)
    }
}

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
    a: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xff }
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: f64) -> Self {
        Self {
            r,
            g,
            b,
            a: (a * 255.0) as u8,
        }
    }

    pub fn from_rgba_u32(c: u32) -> Self {
        Self::from_rgba(
            ((c >> 16) & 0xff) as u8,
            ((c >> 8) & 0xff) as u8,
            (c & 0xff) as u8,
            ((c >> 24) & 0xff) as f64 / 255.0,
        )
    }

    pub fn from_rgb_u32(c: u32) -> Self {
        Self::from_rgb(
            ((c >> 16) & 0xff) as u8,
            ((c >> 8) & 0xff) as u8,
            (c & 0xff) as u8,
        )
    }

    pub fn with_alpha(&self, a: f64) -> Self {
        Self::from_rgba(self.r, self.g, self.b, a)
    }

    pub fn set(&self, ctx: &Context) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        if self.a == 0xff {
            ctx.set_source_rgb(r, g, b);
        } else {
            ctx.set_source_rgba(r, g, b, self.a as f64 / 255.0);
        }
    }

    pub fn luminance(&self) -> f64 {
        let r = self.r as f64 / 256.0;
        let g = self.g as f64 / 256.0;
        let b = self.b as f64 / 256.0;
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn as_f64(&self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }

    pub fn r(&self) -> u8 {
        self.r
    }

    pub fn r_f64(&self) -> f64 {
        self.r as f64 / 255.0
    }

    pub fn g(&self) -> u8 {
        self.g
    }

    pub fn g_f64(&self) -> f64 {
        self.g as f64 / 255.0
    }

    pub fn b(&self) -> u8 {
        self.b
    }

    pub fn b_f64(&self) -> f64 {
        self.b as f64 / 255.0
    }

    pub fn alpha(&self) -> f64 {
        self.a as f64 * 255.0
    }

    pub fn brighter(&self, k: f64) -> Self {
        let (r, g, b) = self.as_f64();
        let k = BRIGHTER.powf(k);
        Self {
            a: self.a,
            r: (r * 255.0 * k) as u8,
            g: (g * 255.0 * k) as u8,
            b: (b * 255.0 * k) as u8,
        }
    }

    pub fn darker(&self, k: f64) -> Self {
        let (r, g, b) = self.as_f64();
        let k = DARKER.powf(k);
        Self {
            a: self.a,
            r: (r * 255.0 * k) as u8,
            g: (g * 255.0 * k) as u8,
            b: (b * 255.0 * k) as u8,
        }
    }

    pub fn white() -> Self {
        Self::from_rgb(0xff, 0xff, 0xff)
    }

    pub fn black() -> Self {
        Self::from_rgb(0x00, 0x00, 0x00)
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
            colors.push(Color::from_rgb_u32(BigEndian::read_u32(
                &self.mem[b..b + 4],
            )));
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
