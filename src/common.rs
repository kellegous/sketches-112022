use crate::{a, b, c, d, e, RenderOpts};
use cairo::Context;
use chrono::Utc;
use clap::{Subcommand, ValueEnum};
use std::{error::Error, fmt::Display};

#[derive(Subcommand, Debug)]
pub enum Command {
    A(a::Args),
    B,
    C,
    D,
    E(e::Args),
}

impl Command {
    pub fn render(&self, args: &dyn RenderOpts, ctx: &Context) -> Result<(), Box<dyn Error>> {
        match self {
            Command::A(params) => a::render(args, ctx, params),
            Command::B => b::render(args, ctx),
            Command::C => c::render(args, ctx),
            Command::D => d::render(args, ctx),
            Command::E(params) => e::render(args, ctx, params),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Command::A(_) => "a",
            Command::B => "b",
            Command::C => "c",
            Command::D => "d",
            Command::E(_) => "e",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Png,
    Pdf,
}

impl Format {
    pub fn extension(&self) -> &str {
        match self {
            Format::Pdf => "pdf",
            Format::Png => "png",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Seed {
    v: u64,
}

impl Default for Seed {
    fn default() -> Self {
        Self {
            v: Utc::now().timestamp() as u64,
        }
    }
}

impl Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.v)
    }
}

impl Seed {
    pub fn new(v: u64) -> Self {
        Self { v }
    }

    pub fn from_arg(s: &str) -> Result<Seed, String> {
        u64::from_str_radix(s, 16)
            .map(Seed::new)
            .map_err(|_| format!("invalid seed: {}", s))
    }
}
