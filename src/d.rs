use crate::{Color, RenderOpts};
use cairo::Context;
use rand::Rng;
use std::error::Error;

struct Series {
    width: f64,
    height: f64,
    pts: Vec<(f64, f64)>,
}

impl Series {
    fn new(width: f64, height: f64, pts: Vec<(f64, f64)>) -> Self {
        Self { width, height, pts }
    }
}

fn color_contrasting_with(c: &Color) -> Color {
    if c.luminance() > 0.5 {
        Color::from_rgb(0x33, 0x33, 0x33)
    } else {
        Color::white()
    }
}

fn draw_path(ctx: &Context, pts: &[(f64, f64)], width: f64) -> Result<(), Box<dyn Error>> {
    ctx.new_path();
    let &(_, y) = pts.first().ok_or_else(|| String::from("empty path"))?;
    ctx.move_to(0.0, y);
    for &(x, y) in pts.iter() {
        ctx.line_to(x, y);
    }
    let &(_, y) = pts.last().unwrap();
    ctx.line_to(width, y);

    Ok(())
}

pub fn render(opts: &dyn RenderOpts, ctx: &Context) -> Result<(), Box<dyn Error>> {
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

    let nx = rng.gen_range(20..80);
    let ny = rng.gen_range(5..20);
    let dx = width / nx as f64;
    let dy = height / ny as f64;

    let c = color_contrasting_with(&theme[0]);

    let pairs = (0..nx)
        .map(|i| {
            let ya = rng.gen_range(0..ny - 1);
            (ya, rng.gen_range(ya + 1..ny))
        })
        .collect::<Vec<_>>();

    let path_a = pairs
        .iter()
        .enumerate()
        .map(|(i, &(y, _))| (i as f64 * dx + dx / 2.0, y as f64 * dy + dy / 2.0))
        .collect::<Vec<_>>();

    let path_b = pairs
        .iter()
        .enumerate()
        .map(|(i, &(_, y))| (i as f64 * dx + dx / 2.0, y as f64 * dy + dy / 2.0))
        .collect::<Vec<_>>();

    ctx.save()?;
    ctx.new_path();
    for i in 0..nx {
        let x = i as f64 * dx + dx / 2.0;
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
    }
    c.set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.new_path();
    for i in 0..ny {
        let y = i as f64 * dy + dy / 2.0;
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
    }
    c.set(ctx);
    ctx.set_dash(&[2.0, 3.0], 0.0);
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.set_line_width(4.0);
    draw_path(ctx, &path_a, width)?;
    theme[1].set(ctx);
    ctx.stroke()?;
    theme[2].set(ctx);
    draw_path(ctx, &path_b, width)?;
    ctx.stroke()?;
    ctx.restore()?;

    // ctx.save()?;
    // ctx.set_line_width(4.0);
    // ctx.new_path();
    // for (i, (ya, _)) in pairs.iter().enumerate() {
    //     let x = i as f64 * dx + dx / 2.0;
    //     let y = *ya as f64 * dy + dy / 2.0;
    //     if i == 0 {
    //         ctx.move_to(x, y);
    //     } else {
    //         ctx.line_to(x, y);
    //     }
    // }
    // theme[1].set(ctx);
    // ctx.stroke()?;
    // ctx.new_path();
    // for (i, (_, yb)) in pairs.iter().enumerate() {
    //     let x = i as f64 * dx + dx / 2.0;
    //     let y = *yb as f64 * dy + dy / 2.0;
    //     if i == 0 {
    //         ctx.move_to(x, y);
    //     } else {
    //         ctx.line_to(x, y);
    //     }
    // }
    // theme[2].set(ctx);
    // ctx.stroke()?;
    // ctx.restore()?;

    Ok(())
}
