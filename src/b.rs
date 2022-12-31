use super::{Color, Rect, RenderOpts};
use cairo::{Context, LineCap, RadialGradient};
use rand::Rng;
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

fn render_bg(
    ctx: &Context,
    ca: &Color,
    cb: &Color,
    r: f64,
    bounds: &Rect,
) -> Result<(), Box<dyn Error>> {
    let g = RadialGradient::new(0.0, 0.0, 0.0, 0.0, 0.0, r);
    g.add_color_stop_rgb(0.0, ca.r_f64(), ca.g_f64(), ca.b_f64());
    g.add_color_stop_rgb(1.0, cb.r_f64(), cb.g_f64(), cb.b_f64());
    ctx.set_source(g)?;
    ctx.new_path();
    ctx.rectangle(bounds.x(), bounds.y(), bounds.width(), bounds.height());
    ctx.fill()?;
    Ok(())
}

pub fn render(opts: &dyn RenderOpts, ctx: &Context) -> Result<(), Box<dyn Error>> {
    let size = opts.size();
    let width = size.width() as f64;
    let height = size.height() as f64;

    let mut rng = opts.rng();

    let themes = opts.themes()?;
    let (_, theme) = themes.pick(&mut rng);

    let cx = width / 2.0;
    let cy = height / 2.0;

    let r = cx.min(cy);
    let ri = r * 0.4;
    let ro = ri * (1.1 + rng.gen::<f64>() * 0.4);

    ctx.save()?;
    let ca = &theme[0];
    let cb = if ca.luminance() > 0.5 {
        ca.darker(1.0)
    } else {
        ca.brighter(1.5)
    };
    ctx.translate(cx, cy);
    render_bg(ctx, ca, &cb, cx.max(cy), &Rect::from_ltrb(-cx, -cy, cx, cy))?;
    ctx.restore()?;

    let rt = rng.gen_range(r * 0.02..r * 0.1);
    let n = 2 * rng.gen_range(4..10);

    ctx.save()?;
    ctx.translate(cx, cy);
    for i in (0..=3).rev() {
        let o = rt * i as f64;
        theme[i + 1].set(ctx);
        burst_path(ctx, ro + o, ri + o, n);
        ctx.fill()?;
    }
    ctx.restore()?;

    ctx.save()?;
    ctx.translate(cx, cy);
    for i in 0..=3 {
        let o = rt / 2.0 * i as f64;
        theme[i + 1].set(ctx);
        ctx.arc(0.0, 0.0, ri - o, 0.0, TAU);
        ctx.fill()?;
    }
    ctx.restore()?;

    let lw = rng.gen_range(2.0..6.0);
    ctx.save()?;
    ctx.translate(cx, cy);
    let y_spacing = rng.gen_range(10.0..(height / 8.0));
    ctx.set_line_cap(LineCap::Round);
    theme[3].set(ctx);
    ctx.set_line_width(lw);
    tendrils_path(
        ctx,
        ri + 4.5 * rt,
        r,
        y_spacing,
        &Rect::from_xywh(-cx, -cy, width, height),
        n,
    );
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.translate(cx, cy);
    let dt = TAU / n as f64;
    let t0 = TAU / 4.0;
    let ra = ri + 4.5 * rt;
    let rb = rt / 2.0;
    ctx.set_line_width(lw * 0.9);
    for i in 0..n {
        let t = dt * i as f64 - t0;
        ctx.new_path();
        ctx.arc(ra * t.cos(), ra * t.sin(), rb, 0.0, TAU);
        theme[1].set(ctx);
        ctx.fill_preserve()?;
        theme[3].set(ctx);
        ctx.stroke()?;
    }
    ctx.restore()?;

    ctx.save()?;
    ctx.translate(cx, cy);
    theme[0].set(ctx);
    ctx.arc(0.0, 0.0, ri - 4.0 * rt / 2.0, 0.0, TAU);
    ctx.fill()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.translate(cx, cy);
    ctx.set_line_width(lw * 0.9);
    ctx.new_path();
    ctx.arc(0.0, 0.0, rb, 0.0, TAU);
    theme[1].set(ctx);
    ctx.fill_preserve()?;
    theme[3].set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    Ok(())
}
