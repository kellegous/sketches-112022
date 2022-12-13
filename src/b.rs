use cairo::Context;
use rand::{Rng, RngCore};
use sketches::{Rect, RenderOpts};
use std::{error::Error, f64::consts::PI};

const TAU: f64 = 2.0 * PI;

fn burst_path(ctx: &Context, ro: f64, ri: f64, n: usize) {
    let dt = TAU / n as f64;
    let ot = TAU / 4.0;

    ctx.new_path();
    ctx.move_to(0.0, -ri);
    for i in 1..=n {
        let tb = dt * i as f64 - ot;
        let ta = tb - dt / 2.0;
        ctx.line_to(ro * ta.cos(), ro * ta.sin());
        ctx.line_to(ri * tb.cos(), ri * tb.sin());
    }
    ctx.close_path();
}

fn tendrils_path(ctx: &Context, ri: f64, ro: f64, y_spacing: f64, bounds: &Rect, n: usize) {
    let dt = TAU / n as f64;
    let ot = TAU / 4.0;
    let nh = n / 2;
    for i in 0..=nh {
        let t = dt * i as f64 - ot;
        ctx.move_to(ri * t.cos(), ri * t.sin());
        let y = -(nh as f64) * y_spacing / 2.0 + y_spacing * i as f64;
        ctx.curve_to(
            1.5 * ro * t.cos(),
            1.5 * ro * t.sin(),
            bounds.right() - ro,
            y,
            bounds.right(),
            y,
        );
    }
    for i in 0..=nh {
        let t = 0.75 * TAU - dt * i as f64;
        ctx.move_to(ri * t.cos(), ri * t.sin());
        let y = y_spacing * i as f64 - nh as f64 * y_spacing / 2.0;
        ctx.curve_to(
            1.5 * ro * t.cos(),
            1.5 * ro * t.sin(),
            bounds.x() + ro,
            y,
            bounds.x(),
            y,
        );
    }
}

pub fn render(opts: &dyn RenderOpts, ctx: &Context) -> Result<(), Box<dyn Error>> {
    let size = opts.size();
    let width = size.width() as f64;
    let height = size.height() as f64;

    let mut rng = opts.rng();

    let themes = opts.themes()?;
    let (_, theme) = themes.pick(&mut rng);

    theme[0].set(ctx);
    ctx.rectangle(0.0, 0.0, width, height);
    ctx.fill()?;

    let cx = width / 2.0;
    let cy = height / 2.0;

    let r = cx.min(cy);
    let ri = r * 0.4;
    let ro = ri * (1.1 + rng.gen::<f64>() * 0.4);

    let rt = rng.gen_range(r * 0.02..r * 0.1);
    let radii = (0..=2)
        .rev()
        .map(|i| {
            let o = rt * i as f64;
            (ro + o, ri + o)
        })
        .collect::<Vec<_>>();

    let n = 2 * rng.gen_range(4..10);

    ctx.save()?;
    ctx.translate(cx, cy);
    for (i, (ro, ri)) in radii.iter().enumerate() {
        theme[i + 2].set(ctx);
        burst_path(ctx, *ro, *ri, n);
        ctx.fill()?;
    }
    ctx.restore()?;

    ctx.save()?;
    theme[3].set(ctx);
    ctx.translate(cx, cy);
    tendrils_path(
        ctx,
        radii[0].1 + rt,
        r,
        rng.gen_range(10..40) as f64,
        &Rect::from_xywh(-cx, -cy, width, height),
        n,
    );
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    theme[1].set(ctx);
    ctx.new_path();
    ctx.arc(cx, cy, ri, 0.0, TAU);
    ctx.fill()?;
    ctx.restore()?;

    Ok(())
}
