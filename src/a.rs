use cairo::Context;
use rand::{Rng, RngCore};
use sketches::{Color, RenderOpts};
use std::{error::Error, ops::Index};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, default_value_t = false)]
    show_grid: bool,
}

fn select_colors(rng: &mut dyn RngCore, colors: &[Color]) -> (Color, Vec<Color>) {
    (
        colors[0],
        colors[1..].iter().map(|c| *c).collect::<Vec<_>>(),
    )
    // let iter = colors
    //     .iter()
    //     .enumerate()
    //     .map(|(ix, c)| (ix, *c, c.luminance()));
    // let (ix, bg, _) = if rng.gen::<bool>() {
    //     iter.max_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap())
    // } else {
    //     iter.min_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap())
    // }
    // .unwrap();

    // let colors = colors
    //     .iter()
    //     .enumerate()
    //     .filter(|(i, _)| *i != ix)
    //     .map(|(_, c)| *c)
    //     .collect::<Vec<_>>();

    // (bg, colors)
}

#[derive(Debug)]
struct Series {
    width: f64,
    height: f64,
    pts: Vec<(f64, f64)>,
}

impl Series {
    fn new(width: f64, height: f64, pts: Vec<(f64, f64)>) -> Self {
        Series { width, height, pts }
    }

    fn translate(&self, dx: f64, dy: f64) -> Series {
        Series::new(
            self.width,
            self.height,
            self.pts
                .iter()
                .map(|(x, y)| (*x + dx, *y + dy))
                .collect::<Vec<_>>(),
        )
    }

    fn gen_on_grid(rng: &mut dyn RngCore, grid: &Grid, width: f64, height: f64) -> Series {
        let dw = width / grid.nw() as f64;
        let dh = height / grid.nh() as f64;

        let ow = dw / 2.0;
        let oh = dh / 2.0;

        Series::new(
            width,
            height,
            (0..grid.nw())
                .map(|i| {
                    let j = rng.gen_range(0..grid.nh());
                    (ow + dw * i as f64, oh + dh * j as f64)
                })
                .collect::<Vec<_>>(),
        )
    }

    fn stroke(&self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        ctx.new_path();
        let (x, y) = self.pts.first().unwrap();
        ctx.move_to(0.0, *y);
        ctx.line_to(*x, *y);

        for i in 1..self.pts.len() {
            let (xa, ya) = self.pts[i - 1];
            let (xb, yb) = self.pts[i];
            let dx = xb - xa;

            let ow = dx / 2.0;
            let c = ow * 0.5523;
            if yb - ya > dx {
                ctx.curve_to(xa + c, ya, xa + ow, ya + ow - c, xa + ow, ya + ow);
                ctx.line_to(xb - ow, yb - ow);
                ctx.curve_to(xb - ow, yb - ow + c, xb - c, yb, xb, yb);
            } else if ya - yb > dx {
                ctx.curve_to(xa + c, ya, xa + ow, ya - ow + c, xa + ow, ya - ow);
                ctx.line_to(xb - ow, yb + ow);
                ctx.curve_to(xb - ow, yb + ow - c, xb - c, yb, xb, yb);
            } else {
                ctx.curve_to(xa + ow, ya, xb - ow, yb, xb, yb);
            }
        }

        let (_, y) = self.pts.last().unwrap();
        ctx.line_to(self.width, *y);
        ctx.stroke()?;

        Ok(())
    }

    fn render(&self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        ctx.new_path();
        let (x, y) = self.pts.first().unwrap();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(0.0, *y);
        ctx.line_to(*x, *y);

        for i in 1..self.pts.len() {
            let (xa, ya) = self.pts[i - 1];
            let (xb, yb) = self.pts[i];

            let dx = xb - xa;

            let ow = dx / 2.0;
            let c = ow * 0.5523;

            if yb - ya > dx {
                ctx.curve_to(xa + c, ya, xa + ow, ya + ow - c, xa + ow, ya + ow);
                ctx.line_to(xb - ow, yb - ow);
                ctx.curve_to(xb - ow, yb - ow + c, xb - c, yb, xb, yb);
            } else if ya - yb > dx {
                ctx.curve_to(xa + c, ya, xa + ow, ya - ow + c, xa + ow, ya - ow);
                ctx.line_to(xb - ow, yb + ow);
                ctx.curve_to(xb - ow, yb + ow - c, xb - c, yb, xb, yb);
            } else {
                ctx.curve_to(xa + ow, ya, xb - ow, yb, xb, yb);
            }
        }

        let (_, y) = self.pts.last().unwrap();
        ctx.line_to(self.width, *y);
        ctx.line_to(self.width, 0.0);
        ctx.close_path();
        ctx.fill()?;

        Ok(())
    }
}

impl Index<usize> for Series {
    type Output = (f64, f64);
    fn index(&self, ix: usize) -> &Self::Output {
        self.pts.index(ix)
    }
}

struct Grid {
    nw: i32,
    nh: i32,
}

impl Grid {
    fn new(nw: i32, nh: i32) -> Grid {
        Grid { nw, nh }
    }

    fn nw(&self) -> i32 {
        self.nw
    }

    fn nh(&self) -> i32 {
        self.nh
    }
}

fn render_grid(
    ctx: &Context,
    color: Color,
    width: f64,
    height: f64,
    grid: &Grid,
) -> Result<(), Box<dyn Error>> {
    color.with_alpha(0.6).set(ctx);

    let nw = grid.nw();
    let nh = grid.nh();

    let dw = width / nw as f64;
    let dh = height / nh as f64;

    ctx.save()?;
    ctx.new_path();
    ctx.translate(dw / 2.0, 0.0);
    for i in 0..nw {
        let x = dw * i as f64;
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
    }
    ctx.set_line_width(1.0);
    ctx.set_dash(&vec![1.0, 4.0], 0.0);
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.new_path();
    ctx.translate(0.0, dh / 2.0);
    for i in 0..nh {
        let y = dh * i as f64;
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
    }
    ctx.set_line_width(1.0);
    ctx.set_dash(&vec![1.0, 4.0], 0.0);
    ctx.stroke()?;
    ctx.restore()?;

    Ok(())
}

fn render_series(
    ctx: &Context,
    series: &Series,
    ca: Color,
    cb: Color,
    line_width: f64,
) -> Result<(), Box<dyn Error>> {
    ctx.save()?;

    ca.set(ctx);
    series.render(ctx)?;

    cb.set(ctx);
    ctx.set_line_width(line_width);
    series.stroke(ctx)?;
    ctx.restore()?;
    Ok(())
}

pub fn render(opts: &dyn RenderOpts, ctx: &Context, args: &Args) -> Result<(), Box<dyn Error>> {
    let size = opts.size();
    let width = size.width() as f64;
    let height = size.height() as f64;

    let mut rng = opts.rng();

    let themes = opts.themes()?;
    let (_, theme) = themes.pick(&mut rng);
    let (bg, theme) = select_colors(&mut rng, &theme);

    bg.set(ctx);
    ctx.rectangle(0.0, 0.0, width, height);
    ctx.fill()?;

    let grid = Grid::new(rng.gen_range(5..40), rng.gen_range(5..40));
    render_series(
        ctx,
        &Series::gen_on_grid(&mut rng, &grid, width, height),
        theme[0],
        theme[1],
        20.0,
    )?;

    render_series(
        ctx,
        &Series::gen_on_grid(&mut rng, &grid, width, height),
        theme[2],
        theme[3],
        20.0,
    )?;

    if args.show_grid {
        let color = if bg.luminance() > 0.5 {
            Color::black()
        } else {
            Color::white()
        };

        render_grid(ctx, color, width, height, &grid)?;
    }

    Ok(())
}
