use crate::RenderOpts;
use cairo::Context;
use rand::Rng;
use std::error::Error;

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

    ctx.save()?;
    ctx.new_path();
    for i in 0..nx {
        let x = i as f64 * dx + dx / 2.0;
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
    }
    theme[1].set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    ctx.new_path();
    for i in 0..ny {
        let y = i as f64 * dy + dy / 2.0;
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
    }
    theme[1].set(ctx);
    ctx.set_dash(&[2.0, 3.0], 0.0);
    ctx.stroke()?;
    ctx.restore()?;

    Ok(())
}
