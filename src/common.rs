use crate::{a, b, c, RenderOpts};
use cairo::Context;
use clap::{Subcommand, ValueEnum};
use std::error::Error;

#[derive(Subcommand, Debug)]
pub enum Command {
    A(a::Args),
    B,
    C,
}

impl Command {
    pub fn render(&self, args: &dyn RenderOpts, ctx: &Context) -> Result<(), Box<dyn Error>> {
        match self {
            Command::A(params) => a::render(args, ctx, params),
            Command::B => b::render(args, ctx),
            Command::C => c::render(args, ctx),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Command::A(_) => "a",
            Command::B => "b",
            Command::C => "c",
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
