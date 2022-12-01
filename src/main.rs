use cairo::{Context, ImageSurface, PdfSurface};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use rand::prelude::*;
use rand_pcg::Pcg64;
use sketches::{RenderOpts, Size, Themes};
use std::{error::Error, fs, io, path::PathBuf};

mod a;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t=Utc::now().timestamp() as u64)]
    seed: u64,

    #[arg(long, default_value_t=Size::new(1600, 600), value_parser=Size::from_arg)]
    size: Size,

    #[arg(long, default_value_t=String::from("themes.bin"))]
    themes: String,

    #[arg(long, value_enum, default_value_t=Format::Png)]
    format: Format,

    #[arg(long)]
    dest: Option<String>,

    #[command(subcommand)]
    command: Command,
}

impl Args {
    fn dest(&self) -> PathBuf {
        let dest = match &self.dest {
            Some(v) => v.as_str(),
            None => self.command.name(),
        };
        PathBuf::from(format!("{}.{}", dest, self.format.extension()))
    }
}

impl RenderOpts for Args {
    fn size(&self) -> Size {
        self.size
    }

    fn rng(&self) -> Pcg64 {
        Pcg64::seed_from_u64(self.seed)
    }

    fn themes(&self) -> io::Result<Themes> {
        Themes::open(&self.themes)
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    A(a::Args),
}

impl Command {
    fn render(&self, args: &Args, ctx: &Context) -> Result<(), Box<dyn Error>> {
        match self {
            Command::A(params) => a::render(args, ctx, params),
        }
    }

    fn name(&self) -> &str {
        match self {
            Command::A(_) => "a",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    Png,
    Pdf,
}

impl Format {
    fn render(&self, args: &Args) -> Result<(), Box<dyn Error>> {
        let size = args.size();
        match args.format {
            Format::Pdf => {
                let surface =
                    PdfSurface::new(size.width() as f64, size.height() as f64, args.dest())?;
                let ctx = Context::new(&surface)?;
                args.command.render(args, &ctx)?;
                surface.finish();
                Ok(())
            }
            Format::Png => {
                let surface =
                    ImageSurface::create(cairo::Format::ARgb32, size.width(), size.height())?;
                let ctx = Context::new(&surface)?;
                args.command.render(args, &ctx)?;
                surface.write_to_png(&mut fs::File::create(args.dest())?)?;
                Ok(())
            }
        }
    }

    fn extension(&self) -> &str {
        match self {
            Format::Pdf => "pdf",
            Format::Png => "png",
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    args.format.render(&args)
}
