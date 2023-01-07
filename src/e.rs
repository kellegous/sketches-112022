use crate::{Color, RenderOpts};
use cairo::{Context, LineCap};
use rand::Rng;
use std::{error::Error, f64::consts::PI, ops::Range};

const TAU: f64 = 2.0 * PI;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, default_value_t = false)]
    show_grid: bool,

    #[arg(long, default_value_t = true)]
    show_halos: bool,
}

fn index_of_max(colors: &[Color]) -> usize {
    colors
        .iter()
        .map(|c| c.luminance())
        .enumerate()
        .max_by(|(_, la), (_, lb)| la.total_cmp(lb))
        .unwrap()
        .0
}

fn index_of_min(colors: &[Color]) -> usize {
    colors
        .iter()
        .map(|c| c.luminance())
        .enumerate()
        .min_by(|(_, la), (_, lb)| la.total_cmp(lb))
        .unwrap()
        .0
}

fn segment_theme(rng: &mut dyn rand::RngCore, theme: &[Color]) -> (Color, Color, Vec<Color>) {
    let min = index_of_min(theme);
    let max = index_of_max(theme);
    let rest = theme
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != max && i != min)
        .map(|(_, c)| *c)
        .collect::<Vec<_>>();
    if rng.gen::<bool>() {
        (theme[min], theme[max], rest)
    } else {
        (theme[max], theme[min], rest)
    }
}

fn select_nodes(
    rng: &mut dyn rand::RngCore,
    grid: &Grid,
    colors: &[Color],
    density: f64,
) -> Vec<Vec<(Color, usize)>> {
    let mut nodes = Vec::new();
    for _ in grid.x_range() {
        let mut picks = Vec::new();
        // we need to pick at least one
        let j = rng.gen_range(0..grid.ny / 2);
        picks.push((colors[0], j));
        for j in j + 1..grid.ny {
            if rng.gen::<f64>() < density {
                picks.push((colors[picks.len() % colors.len()], j))
            }
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

fn pick<T>(rng: &mut dyn rand::RngCore, a: T, b: T) -> T {
    if rng.gen::<bool>() {
        a
    } else {
        b
    }
}

struct Path {
    pts: Vec<(f64, f64)>,
}

impl Path {
    #[allow(dead_code)]
    fn draw(&self, ctx: &Context) {
        let &(x, y) = self.pts.first().unwrap();
        ctx.move_to(x, y);
        for &(x, y) in &self.pts[1..] {
            ctx.line_to(x, y);
        }
    }

    fn draw_smooth(&self, ctx: &Context) {
        let &(x, y) = self.pts.first().unwrap();
        ctx.move_to(x, y);
        for i in 1..self.pts.len() {
            let (xa, ya) = self.pts[i - 1];
            let (xb, yb) = self.pts[i];
            if (xa - xb).abs() < 0.001 {
                ctx.line_to(xb, yb);
            } else {
                let cy = (yb - ya) / 2.0;
                ctx.curve_to(xa, ya + cy, xb, ya + cy, xb, yb);
            }
        }
    }

    fn first(&self) -> &(f64, f64) {
        self.pts.first().unwrap()
    }

    fn last(&self) -> &(f64, f64) {
        self.pts.last().unwrap()
    }
}

fn build_vline(
    rng: &mut dyn rand::RngCore,
    grid: &Grid,
    r: f64,
    x: f64,
    nodes: &[(Color, usize)],
) -> Path {
    let dyh = grid.dy / 2.0;

    let mut pts = Vec::new();
    let mut cx = x + pick(rng, -r, r);
    let &(_, j) = nodes.first().unwrap();
    if j == 0 {
        pts.push((cx, dyh));
        pts.push((cx, grid.y_of(j)));
    } else {
        let y = grid.y_of(j);
        pts.push((x, dyh));
        pts.push((x, y - grid.dy));
        pts.push((cx, y));
    }

    for k in 1..nodes.len() {
        let (_, ja) = nodes[k - 1];
        let (_, jb) = nodes[k];
        if jb - ja == 1 {
            pts.push((cx, grid.y_of(jb)));
        } else {
            let ya = grid.y_of(ja);
            let yb = grid.y_of(jb);
            pts.push((x, ya + grid.dy));
            pts.push((x, yb - grid.dy));
            cx = x + pick(rng, -r, r);
            pts.push((cx, yb));
        }
    }

    let &(_, j) = nodes.last().unwrap();
    if j == grid.ny - 1 {
        pts.push((cx, grid.y_of(grid.ny - 1) + dyh));
    } else {
        let y = grid.y_of(j);
        pts.push((x, y + grid.dy));
        pts.push((x, grid.y_of(grid.ny - 1) + dyh));
    }
    Path { pts }
}

fn shadow_over(base: &Color, alpha: f64) -> Color {
    if base.luminance() > 0.5 {
        Color::black()
    } else {
        Color::white()
    }
    .with_alpha(alpha)
}

pub fn render(opts: &dyn RenderOpts, ctx: &Context, args: &Args) -> Result<(), Box<dyn Error>> {
    let size = opts.size();
    let width = size.width() as f64;
    let height = size.height() as f64;

    let mut rng = opts.rng();

    let shadow_dx = 3.0;
    let shadow_dy = 2.0;

    let themes = opts.themes()?;
    let (_, theme) = themes.pick(&mut rng);

    let (ca, cb, colors) = segment_theme(&mut rng, &theme);

    ctx.save()?;
    ctx.new_path();
    ctx.rectangle(0.0, 0.0, width, height);
    ca.set(ctx);
    ctx.fill()?;
    ctx.restore()?;

    let grid = Grid::new(width, height, rng.gen_range(20..40), rng.gen_range(5..20));

    if args.show_grid {
        ctx.save()?;
        grid.render(ctx)?;
        cb.set(ctx);
        ctx.set_dash(&[1.0, 4.0], 0.0);
        ctx.set_line_width(1.0);
        ctx.stroke()?;
        ctx.restore()?;
    }

    let r = grid.dx.min(grid.dy);
    let ra = r / 5.0;
    let rb = r * 0.45;
    let nodes = select_nodes(&mut rng, &grid, &colors, 0.25);
    let paths = nodes
        .iter()
        .enumerate()
        .map(|(i, nodes)| build_vline(&mut rng, &grid, rb, grid.x_of(i), nodes))
        .collect::<Vec<_>>();

    ctx.save()?;
    ctx.translate(shadow_dx, shadow_dy);
    paths.iter().for_each(|p| p.draw_smooth(ctx));
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Round);
    shadow_over(&ca, 0.2).set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    ctx.save()?;
    paths.iter().for_each(|p| p.draw_smooth(ctx));
    ctx.set_line_width(4.0);
    ctx.set_line_cap(LineCap::Round);
    cb.set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    // cap the ends of the vlines
    ctx.save()?;
    cb.set(ctx);
    for path in paths.iter() {
        let &(x, y) = path.first();
        ctx.new_path();
        ctx.arc(x, y, 4.0, 0.0, TAU);
        ctx.fill()?;

        let &(x, y) = path.last();
        ctx.new_path();
        ctx.arc(x, y, 4.0, 0.0, TAU);
        ctx.fill()?;
    }
    ctx.restore()?;

    // draw node shadows
    ctx.save()?;
    ctx.translate(shadow_dx + 1.0, shadow_dy + 1.0);
    for (i, nodes) in nodes.iter().enumerate() {
        for &(_, j) in nodes.iter() {
            ctx.new_path();
            ctx.arc(grid.x_of(i), grid.y_of(j), ra, 0.0, TAU);
            shadow_over(&ca, 0.2).set(ctx);
            ctx.fill()?;
        }
    }
    ctx.restore()?;

    // draw the nodes
    ctx.save()?;
    ctx.set_line_width(4.0);
    for (i, nodes) in nodes.iter().enumerate() {
        for (color, j) in nodes.iter() {
            let x = grid.x_of(i);
            let y = grid.y_of(*j);

            ctx.new_path();
            ctx.arc(x, y, ra, 0.0, TAU);
            color.set(ctx);
            ctx.fill()?;

            ctx.set_line_width(4.0);
            ctx.new_path();
            ctx.arc(x, y, ra - 1.0, 0.0, TAU);
            shadow_over(color, 0.2).set(ctx);
            ctx.stroke()?;

            ctx.new_path();
            ctx.arc(x, y, ra, 0.0, TAU);
            cb.set(ctx);
            ctx.stroke()?;
        }
    }
    ctx.restore()?;

    if args.show_halos {
        ctx.save()?;
        for (i, nodes) in nodes.iter().enumerate() {
            for (_, j) in nodes.iter() {
                let x = grid.x_of(i);
                let y = grid.y_of(*j);
                ctx.new_path();
                ctx.arc(x, y, r, 0.0, TAU);
                shadow_over(&ca, 1.0).set(ctx);
                ctx.set_line_width(2.0);
                ctx.set_dash(&[1.0, 4.0], 0.0);
                ctx.stroke()?;
            }
        }
        ctx.restore()?;
    }

    Ok(())
}
