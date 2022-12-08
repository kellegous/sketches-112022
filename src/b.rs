use cairo::Context;
use rand::Rng;
use sketches::{Color, RenderOpts};
use std::{error::Error, f64::consts::PI};

const TAU: f64 = 2.0 * PI;

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
    let ri = r * 0.6;
    let ro = ri * (1.1 + rng.gen::<f64>() * 0.4);

    let n = 2 * rng.gen_range(4..10);
    let dt = TAU / n as f64;
    let ot = TAU / 4.0;

    ctx.save()?;
    for i in 0..n {
        let tb = dt * i as f64 - ot;
        let ta = tb - dt / 2.0;
        ctx.line_to(cx + ro * ta.cos(), cy + ro * ta.sin());
        ctx.line_to(cx + ri * tb.cos(), cy + ri * tb.sin());
    }
    ctx.close_path();
    theme[2].set(ctx);
    ctx.set_line_width(1.0);
    ctx.fill()?;
    ctx.restore()?;

    // tendrils
    ctx.save()?;
    theme[3].set(ctx);
    let ds = rng.gen_range(10..40) as f64;
    let nh = n / 2;
    // ctx.set_dash(&[1.0, 5.0], 0.0);
    ctx.set_line_width(2.0);
    for i in 0..=nh {
        let t = dt * i as f64 - TAU / 4.0;
        ctx.move_to(cx + ri * t.cos(), cy + ri * t.sin());
        let y = cy - nh as f64 * ds / 2.0 + ds * i as f64;
        ctx.curve_to(
            cx + 1.5 * r * t.cos(),
            cy + 1.5 * r * t.sin(),
            width - r,
            y,
            width,
            y,
        );
    }
    for i in 0..=nh {
        let t = 0.75 * TAU - dt * i as f64;
        ctx.move_to(cx + ri * t.cos(), cy + ri * t.sin());
        let y = cy - nh as f64 * ds / 2.0 + ds * i as f64;
        ctx.curve_to(cx + 1.5 * r * t.cos(), cy + 1.5 * r * t.sin(), r, y, 0.0, y);
    }
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
