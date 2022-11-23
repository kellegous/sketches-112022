use cairo::Context;
use rand::{Rng, RngCore};
use sketches::{Color, RenderOpts, Size};
use std::f64::consts::PI;
use std::sync::Arc;
use std::{error::Error, ops::Range};

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

fn create_series(
    rng: &mut dyn RngCore,
    width: f64,
    height: f64,
    nw: i32,
    nh: i32,
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let dw = width / nw as f64;
    let dh = height / nh as f64;

    let ow = dw / 2.0;
    let oh = dh / 2.0;

    let mut sa = Vec::with_capacity(2 + nw as usize);
    let mut sb = Vec::with_capacity(2 + nw as usize);

    let ya = rng.gen_range(0..nh);
    let yb = rng.gen_range(ya..nh);
    let ya = oh + dh * ya as f64;
    let yb = oh + dh * yb as f64;

    sa.push((0.0, ya));
    sa.push((ow, ya));
    sb.push((0.0, yb));
    sb.push((ow, yb));

    for i in 1..nw - 1 {
        let x = ow + dw * i as f64;
        let ya = rng.gen_range(0..nh);
        sa.push((x, oh + dh * ya as f64));
        let yb = rng.gen_range(ya..nh);
        sb.push((x, oh + dh * yb as f64));
    }

    let ya = rng.gen_range(0..nh);
    let yb = rng.gen_range(ya..nh);
    let ya = oh + dh * ya as f64;
    let yb = oh + dh * yb as f64;
    sa.push((width - ow, ya));
    sa.push((width, ya));
    sb.push((width - ow, yb));
    sb.push((width, yb));

    (sa, sb)
}

fn render_grid(
    ctx: &Context,
    color: Color,
    width: f64,
    height: f64,
    nw: i32,
    nh: i32,
) -> Result<(), Box<dyn Error>> {
    color.set_with_alpha(ctx, 0.6);

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

    let nw = rng.gen_range(5..40);
    let nh = rng.gen_range(5..40);

    let themes = opts.themes()?;
    let (_, theme) = themes.pick(&mut rng);
    let (bg, theme) = select_colors(&mut rng, &theme);

    bg.set(ctx);
    ctx.rectangle(0.0, 0.0, width, height);
    ctx.fill()?;

    render_grid(
        ctx,
        if bg.luminance() > 0.5 {
            Color::from_u32(0x000000)
        } else {
            Color::from_u32(0xffffff)
        },
        width,
        height,
        nw,
        nh,
    )?;

    let (sa, sb) = create_series(&mut rng, width, height, nw, nh);

    ctx.save()?;
    ctx.new_path();
    ctx.move_to(0.0, 0.0);
    for (x, y) in sa.iter() {
        ctx.line_to(*x, *y);
    }
    ctx.line_to(width, 0.0);
    ctx.close_path();
    theme[0].set_with_alpha(ctx, 0.6);
    ctx.fill()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.new_path();
    ctx.move_to(0.0, height);
    for (x, y) in sb.iter() {
        ctx.line_to(*x, *y);
    }
    ctx.line_to(width, height);
    ctx.close_path();
    theme[1].set_with_alpha(ctx, 0.6);
    ctx.fill()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.set_line_width(1.0);
    for (x, y) in sa.iter() {
        ctx.new_path();
        ctx.arc(*x, *y, 3.0, 0.0, 2.0 * PI);
        Color::from_u32(0xffffff).set(ctx);
        ctx.fill_preserve()?;
        Color::from_u32(0x000000).set(ctx);
        ctx.stroke()?;
    }
    ctx.restore()?;

    // {
    //     let dw = width / nw as f64;
    //     let dh = height / nh as f64;
    //     ctx.save()?;
    //     ctx.translate(dw / 2.0, dh / 2.0);
    //     for i in 0..nw {
    //         let x = dw * i as f64;
    //         let y = dh * rng.gen_range(0..nh) as f64;
    //         ctx.new_path();
    //         ctx.arc(x, y, 5.0, 0.0, 2.0 * PI);
    //         theme[0].set(ctx);
    //         ctx.fill_preserve()?;
    //         theme[2].set(ctx);
    //         ctx.set_line_width(2.0);
    //         ctx.stroke()?;
    //     }
    //     ctx.restore()?;
    // }

    Ok(())
}
