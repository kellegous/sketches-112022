use cairo::{Context, ImageSurface, PdfSurface};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use rand::prelude::*;
use rand_pcg::Pcg64;
use sketches::{RenderOpts, Size, Themes};
use std::{error::Error, fs, io, path::PathBuf};

mod a;
mod b;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, default_value_t=Utc::now().timestamp() as u64)]
    seed: u64,

    #[arg(long, default_value_t=Size::new(1600, 600), value_parser=Size::from_arg)]
    size: Size,

    #[arg(long, default_value_t=String::from("themes.bin"))]
    themes: String,

    #[arg(long, value_enum, default_value_t=Format::Png)]
    format: Format,

    #[arg(long, default_value_t=String::from("{name}.{extension}"))]
    dest: String,

    #[command(subcommand)]
    command: Command,
}

impl Args {
    fn dest(&self) -> Result<PathBuf, Box<dyn Error>> {
        let v = template::render(&self.dest, &template::Context::from_args(self))?;
        Ok(PathBuf::from(v))
    }
    // fn dest(&self) -> PathBuf {
    //     let dest = match &self.dest {
    //         Some(v) => v.as_str(),
    //         None => self.command.name(),
    //     };
    //     PathBuf::from(format!("{}.{}", dest, self.format.extension()))
    // }
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
    B,
}

impl Command {
    fn render(&self, args: &Args, ctx: &Context) -> Result<(), Box<dyn Error>> {
        match self {
            Command::A(params) => a::render(args, ctx, params),
            Command::B => b::render(args, ctx),
        }
    }

    fn name(&self) -> &str {
        match self {
            Command::A(_) => "a",
            Command::B => "b",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    Png,
    Pdf,
}

impl Format {
    #[warn(unused_must_use)]
    fn render(&self, args: &Args) -> Result<(), Box<dyn Error>> {
        let size = args.size();
        let dest = args.dest()?;

        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir).ok();
        }

        match args.format {
            Format::Pdf => {
                let surface = PdfSurface::new(size.width() as f64, size.height() as f64, dest)?;
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
                surface.write_to_png(&mut fs::File::create(dest)?)?;
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

mod template {
    use super::Args;
    use serde::Serialize;
    use std::error::Error;
    use tinytemplate::TinyTemplate;

    #[derive(Serialize)]
    pub struct Context {
        seed: u64,
        name: String,
        extension: String,
    }

    impl Context {
        pub fn from_args(args: &Args) -> Context {
            Context {
                seed: args.seed,
                name: args.command.name().to_owned(),
                extension: args.format.extension().to_owned(),
            }
        }
    }

    pub fn render(tpl: &str, ctx: &Context) -> Result<String, Box<dyn Error>> {
        let mut tt = TinyTemplate::new();
        tt.add_template("t", tpl)?;
        let res = tt.render("t", ctx)?;
        Ok(res)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!("seed: {}", args.seed);
    args.format.render(&args)
}
