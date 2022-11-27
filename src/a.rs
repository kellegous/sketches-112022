use cairo::Context;
use rand::{Rng, RngCore};
use sketches::{Color, RenderOpts, Size};
use std::error::Error;

fn select_colors(rng: &mut dyn RngCore, colors: &[Color]) -> (Color, Vec<Color>) {
    let iter = colors
        .iter()
        .enumerate()
        .map(|(ix, c)| (ix, *c, c.luminance()));
    let (ix, bg, _) = if rng.gen::<bool>() {
        iter.max_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap())
    } else {
        iter.min_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap())
    }
    .unwrap();

    let colors = colors
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != ix)
        .map(|(_, c)| *c)
        .collect::<Vec<_>>();

    (bg, colors)
}

struct Grid {
    nw: i32,
    nh: i32,
}

impl Grid {
    fn new(nw: i32, nh: i32) -> Grid {
        Grid { nw, nh }
    }

    fn gen_series<'a>(&'a self, rng: &'a mut dyn RngCore) -> impl Iterator<Item = (i32, i32)> + '_ {
        (0..self.nw).map(|i| (i, rng.gen_range(0..self.nh)))
    }

    fn nw(&self) -> i32 {
        self.nw
    }

    fn nh(&self) -> i32 {
        self.nh
    }
}

fn render_series(
    ctx: &Context,
    size: &Size,
    grid: &Grid,
    series: &[(i32, i32)],
    color: Color,
) -> Result<(), Box<dyn Error>> {
    let width = size.width() as f64;
    let height = size.height() as f64;

    let dw = width / grid.nw() as f64;
    let dh = height / grid.nh() as f64;

    let ow = dw / 2.0;
    let oh = dh / 2.0;

    ctx.save()?;
    ctx.new_path();
    ctx.move_to(0.0, 0.0);
    let (_, y) = series.first().unwrap();
    ctx.line_to(0.0, oh + dh * *y as f64);
    for (x, y) in series {
        ctx.line_to(ow + dw * *x as f64, oh + dh * *y as f64);
    }

    let (_, y) = series.last().unwrap();
    ctx.line_to(width, oh + dh * *y as f64);
    ctx.line_to(width, 0.0);
    ctx.close_path();
    color.set_with_alpha(ctx, 0.6);
    ctx.fill()?;

    ctx.restore()?;

    Ok(())
}

fn render_grid(
    ctx: &Context,
    color: Color,
    width: f64,
    height: f64,
    grid: &Grid,
) -> Result<(), Box<dyn Error>> {
    color.set_with_alpha(ctx, 0.6);

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

pub fn render(opts: &dyn RenderOpts, ctx: &Context) -> Result<(), Box<dyn Error>> {
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
    render_grid(
        ctx,
        if bg.luminance() > 0.5 {
            Color::from_u32(0x000000)
        } else {
            Color::from_u32(0xffffff)
        },
        width,
        height,
        &grid,
    )?;

    let series = grid.gen_series(&mut rng).collect::<Vec<_>>();
    render_series(ctx, &size, &grid, &series, theme[0])?;

    Ok(())
}
