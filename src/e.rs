use crate::{Color, RenderOpts};
use cairo::{Context, LineCap};
use rand::Rng;
use std::{error::Error, f64::consts::PI, ops::Range};

const TAU: f64 = 2.0 * PI;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, default_value_t = false)]
    show_grid: bool,
}

fn color_contrasting_with(c: &Color) -> Color {
    if c.luminance() > 0.5 {
        Color::from_rgb(0x33, 0x33, 0x33)
    } else {
        Color::white()
    }
}

fn select_nodes(
    rng: &mut dyn rand::RngCore,
    grid: &Grid,
    colors: &[Color],
) -> Vec<Vec<(Color, usize)>> {
    let mut nodes = Vec::new();
    for i in grid.x_range() {
        let mut sum = rng.gen_range(grid.y_range());
        let mut picks = Vec::new();
        while sum < grid.ny && picks.len() < colors.len() {
            picks.push((colors[picks.len()], sum));
            sum += rng.gen_range(grid.y_range());
        }
        nodes.push(picks);
    }
    nodes
}

struct Grid {
    width: f64,
    height: f64,
    nx: usize,
    ny: usize,
    dx: f64,
    dy: f64,
}

impl Grid {
    fn new(width: f64, height: f64, nx: usize, ny: usize) -> Self {
        Self {
            width,
            height,
            nx,
            ny,
            dx: width / (nx + 1) as f64,
            dy: height / (ny + 1) as f64,
        }
    }

    fn x_range(&self) -> Range<usize> {
        0..self.nx
    }

    fn y_range(&self) -> Range<usize> {
        0..self.ny
    }

    fn x_of(&self, i: usize) -> f64 {
        (i + 1) as f64 * self.dx
    }

    fn y_of(&self, j: usize) -> f64 {
        (j + 1) as f64 * self.dy
    }

    fn render(&self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        ctx.new_path();
        for i in 0..self.nx {
            let x = self.x_of(i);
            ctx.move_to(x, 0.0);
            ctx.line_to(x, self.height);
        }

        for j in 0..self.ny {
            let y = self.y_of(j);
            ctx.move_to(0.0, y);
            ctx.line_to(self.width, y);
        }

        Ok(())
    }
}

pub fn render(opts: &dyn RenderOpts, ctx: &Context, args: &Args) -> Result<(), Box<dyn Error>> {
    let size = opts.size();
    let width = size.width() as f64;
    let height = size.height() as f64;

    let mut rng = opts.rng();

    let themes = opts.themes()?;
    let (_, theme) = themes.pick(&mut rng);

    ctx.save()?;
    ctx.new_path();
    ctx.rectangle(0.0, 0.0, width, height);
    theme[0].set(ctx);
    ctx.fill()?;
    ctx.restore()?;

    let c = color_contrasting_with(&theme[0]);

    let grid = Grid::new(width, height, rng.gen_range(20..50), rng.gen_range(5..20));

    if args.show_grid {
        ctx.save()?;
        grid.render(ctx)?;
        c.set(ctx);
        ctx.set_dash(&[1.0, 4.0], 0.0);
        ctx.set_line_width(1.0);
        ctx.stroke()?;
        ctx.restore()?;
    }

    ctx.save()?;
    ctx.new_path();
    for i in grid.x_range() {
        let x = grid.x_of(i);
        ctx.move_to(x, grid.dy / 2.0);
        ctx.line_to(x, height - grid.dy / 2.0);
    }
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Round);
    c.set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    let r = grid.dx.min(grid.dy) / 3.0;
    let nodes = select_nodes(&mut rng, &grid, &theme[1..]);
    ctx.save()?;
    ctx.set_line_width(4.0);
    for (i, nodes) in nodes.iter().enumerate() {
        for (color, j) in nodes.iter() {
            ctx.new_path();
            ctx.arc(grid.x_of(i), grid.y_of(*j), r, 0.0, TAU);
            color.set(ctx);
            ctx.fill_preserve()?;
            c.set(ctx);
            ctx.stroke()?;
        }
    }
    ctx.restore()?;

    Ok(())
}
