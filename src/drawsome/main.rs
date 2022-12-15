use chrono::Utc;
use clap::Parser;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use sketches::{
    common::{Command, Format},
    Size,
};
use std::{error::Error, path::PathBuf, process};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t=Size::new(1600, 600),value_parser=Size::from_arg)]
    size: Size,

    #[arg(long, default_value_t=String::from("themes.bin"))]
    themes: String,

    #[arg(long, value_enum, default_value_t=Format::Png)]
    format: Format,

    #[arg(long, default_value_t = 10)]
    count: usize,

    #[arg(long, default_value_t=String::from("{name}/{seed}.{extension}"))]
    dest: String,

    #[command(subcommand)]
    command: Command,
}

fn bin_dir() -> Result<PathBuf, Box<dyn Error>> {
    let cmd = std::env::args()
        .nth(0)
        .ok_or("could not determine command")?;
    let cmd = PathBuf::from(cmd);
    let dir = cmd.parent().ok_or("no parent directory")?.to_owned();
    Ok(dir)
}

fn main() -> Result<(), Box<dyn Error>> {
    let bin_dir = bin_dir()?;
    let args = Args::parse();

    let mut rng = Pcg64::seed_from_u64(Utc::now().timestamp() as u64);
    for _ in 0..args.count {
        let params = vec![
            format!("--seed={}", rng.gen::<u64>()),
            format!("--size={}", args.size),
            format!("--themes={}", args.themes),
            format!("--format={}", args.format),
            format!("--dest={}", args.dest),
            args.command.name().to_owned(),
        ];

        process::Command::new(bin_dir.join("draw"))
            .args(&params)
            .status()?;
    }

    Ok(())
}
