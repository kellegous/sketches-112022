use cairo::Context;
use rand::{Rng, RngCore};
use sketches::{Color, RenderOpts, Size};
use std::error::Error;
use std::f64::consts::PI;

struct Grid {
    nw: i32,
    nh: i32,
    dw: f64,
    dh: f64,
}

impl Grid {
    fn create(rng: &mut dyn RngCore, size: &Size) -> Grid {
        let nw = rng.gen_range(5..40);
        let nh = rng.gen_range(5..40);
        let dw = size.width() as f64 / nw as f64;
        let dh = size.height() as f64 / nh as f64;
        Grid { nw, nh, dw, dh }
    }
}

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

fn render_grid(
    ctx: &Context,
    width: f64,
    height: f64,
    bg: Color,
    grid: &Grid,
) -> Result<(), Box<dyn Error>> {
    if bg.luminance() > 0.5 {
        Color::from_u32(0x000000)
    } else {
        Color::from_u32(0xffffff)
    }
    .set_with_alpha(ctx, 0.6);

    ctx.save()?;
    ctx.new_path();
    ctx.translate(grid.dw / 2.0, 0.0);
    for i in 0..grid.nw {
        let x = grid.dw * i as f64;
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
    }
    ctx.set_line_width(1.0);
    ctx.set_dash(&vec![1.0, 4.0], 0.0);
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.new_path();
    ctx.translate(0.0, grid.dh / 2.0);
    for i in 0..grid.nh {
        let y = grid.dh * i as f64;
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

    let grid = Grid::create(&mut rng, &size);

    let themes = opts.themes()?;
    let (_, theme) = themes.pick(&mut rng);
    let (bg, theme) = select_colors(&mut rng, &theme);

    bg.set(ctx);
    ctx.rectangle(0.0, 0.0, width, height);
    ctx.fill()?;

    render_grid(ctx, width, height, bg, &grid)?;

    {
        ctx.save()?;
        ctx.translate(grid.dw / 2.0, grid.dh / 2.0);
        for i in 0..grid.nw {
            let x = grid.dw * i as f64;
            let y = grid.dh * rng.gen_range(0..grid.nh) as f64;
            ctx.new_path();
            ctx.arc(x, y, 5.0, 0.0, 2.0 * PI);
            theme[0].set(ctx);
            ctx.fill_preserve()?;
            theme[2].set(ctx);
            ctx.set_line_width(2.0);
            ctx.stroke()?;
        }
        ctx.restore()?;
    }

    Ok(())
}
