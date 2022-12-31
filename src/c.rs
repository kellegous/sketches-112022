use crate::{Color, Point, RenderOpts};
use cairo::Context;
use std::error::Error;

#[derive(Debug, Clone)]
struct Pt3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Pt3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn translate(&self, dx: f64, dy: f64, dz: f64) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    fn scale(&self, sx: f64, sy: f64, sz: f64) -> Self {
        Self::new(self.x * sx, self.y * sy, self.z * sz)
    }

    fn rotate_x(&self, origin: &Pt3, angle: f64) -> Self {
        let y = self.y - origin.y;
        let z = self.z - origin.z;

        let z = z * angle.cos() - y * angle.sin();
        let y = z * angle.sin() + y * angle.cos();

        Self {
            x: self.x,
            y: y + origin.y,
            z: z + origin.z,
        }
    }

    fn rotate_y(&self, origin: &Pt3, angle: f64) -> Self {
        let x = self.x - origin.x;
        let z = self.z - origin.z;

        let x = x * angle.cos() - z * angle.sin();
        let z = x * angle.sin() + z * angle.cos();
        Self {
            x: x + origin.x,
            y: self.y,
            z: z + origin.z,
        }
    }

    fn rotate_z(&self, origin: &Pt3, angle: f64) -> Self {
        let x = self.x - origin.x;
        let y = self.y - origin.y;
        let x = x * angle.cos() - y * angle.sin();
        let y = x * angle.sin() + y * angle.cos();
        Self {
            x: x + origin.x,
            y: y + origin.y,
            z: self.z,
        }
    }

    fn depth(&self) -> f64 {
        self.x + self.y - 2.0 * self.z
    }

    fn distance_between(a: &Pt3, b: &Pt3) -> f64 {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let dz = b.z - a.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

struct Vec3 {
    i: f64,
    j: f64,
    k: f64,
}

impl Vec3 {
    fn new(i: f64, j: f64, k: f64) -> Self {
        Self { i, j, k }
    }

    fn zeros() -> Vec3 {
        Vec3 {
            i: 0.0,
            j: 0.0,
            k: 0.0,
        }
    }
    fn cross_product(a: &Vec3, b: &Vec3) -> Self {
        Self {
            i: a.j * b.k - b.j * a.k,
            j: -1.0 * (a.i * b.k - b.i * a.k),
            k: a.i * b.j - b.i * a.j,
        }
    }

    fn dot_product(a: &Vec3, b: &Vec3) -> f64 {
        a.i * b.i + a.j * b.j + a.k * b.k
    }

    fn magnitude(&self) -> f64 {
        (self.i * self.i + self.j * self.j + self.k * self.k).sqrt()
    }

    fn normalize(&self) -> Vec3 {
        let m = self.magnitude();
        if m == 0.0 {
            Vec3::zeros()
        } else {
            Vec3 {
                i: self.i / m,
                j: self.j / m,
                k: self.k / m,
            }
        }
    }

    fn transform(&self, m: &Mat3) -> Self {
        let ca = m.col(0);
        let cb = m.col(1);
        let cc = m.col(2);
        Self {
            i: ca[0] * self.i + ca[1] * self.j + ca[2] * self.k,
            j: cb[0] * self.i + cb[1] * self.j + cb[2] * self.k,
            k: cc[0] * self.i + cc[1] * self.j + cc[2] * self.k,
        }
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{:0.4}, {:0.4}, {:0.4}}}", self.i, self.j, self.k)
    }
}

struct Mat3 {
    cols: Vec<f64>,
}

impl Mat3 {
    fn new(cols: Vec<f64>) -> Self {
        Self { cols }
    }

    fn col(&self, i: usize) -> &[f64] {
        &self.cols[i * 3..]
    }
}

struct Path {
    pts: Vec<Pt3>,
}

impl Path {
    fn from_points(pts: Vec<Pt3>) -> Self {
        Self { pts }
    }

    fn empty() -> Self {
        Self { pts: vec![] }
    }

    fn add(&mut self, pt: Pt3) {
        self.pts.push(pt)
    }

    fn reverse(&self) -> Self {
        let mut pts = self.pts.clone();
        pts.reverse();
        Self { pts }
    }

    fn translate(&self, dx: f64, dy: f64, dz: f64) -> Self {
        self.map(|p| p.translate(dx, dy, dz))
    }

    fn rotate_x(&self, origin: &Pt3, angle: f64) -> Self {
        self.map(|p| p.rotate_x(origin, angle))
    }

    fn rotate_y(&self, origin: &Pt3, angle: f64) -> Self {
        self.map(|p| p.rotate_y(origin, angle))
    }

    fn rotate_z(&self, origin: &Pt3, angle: f64) -> Self {
        self.map(|p| p.rotate_z(origin, angle))
    }

    fn scale(&self, sx: f64, sy: f64, sz: f64) -> Self {
        self.map(|p| p.scale(sx, sy, sz))
    }

    fn map<F>(&self, op: F) -> Self
    where
        F: Fn(&Pt3) -> Pt3,
    {
        Self {
            pts: self.pts.iter().map(op).collect(),
        }
    }

    fn depth(&self) -> f64 {
        self.pts.iter().map(|p| p.depth()).sum::<f64>() / self.pts.len().max(1) as f64
    }

    fn rectangle(origin: &Pt3, width: f64, height: f64) -> Self {
        Self {
            pts: vec![
                origin.clone(),
                Pt3::new(origin.x + width, origin.y, origin.z),
                Pt3::new(origin.x + width, origin.y + height, origin.z),
                Pt3::new(origin.x, origin.y + height, origin.z),
            ],
        }
    }
}

fn isometric_transform() -> Mat3 {
    let f = 1.0 / f64::sqrt(6.0);
    Mat3::new(vec![
        f64::sqrt(3.0) * f,
        f,
        f64::sqrt(2.0) * f,
        0.0,
        2.0 * f,
        -f64::sqrt(2.0) * f,
        -f64::sqrt(2.0) * f,
        f,
        f64::sqrt(2.0) * f,
    ])
}

fn render_path(ctx: &Context, path: &mut dyn Iterator<Item = Vec3>) -> Result<(), Box<dyn Error>> {
    for (i, pt) in path.enumerate() {
        println!("{}", pt);
        if i == 0 {
            ctx.move_to(pt.i, pt.j);
        } else {
            ctx.line_to(pt.i, pt.j);
        }
    }
    Ok(())
}

fn render_face(
    ctx: &Context,
    path: &[Vec3],
    tx: &Mat3,
    fill_color: &Color,
    stroke_color: &Color,
) -> Result<(), Box<dyn Error>> {
    for (i, pt) in path.iter().enumerate() {
        let pt = pt.transform(tx);
        if i == 0 {
            ctx.move_to(pt.i, pt.j);
        } else {
            ctx.line_to(pt.i, pt.j);
        }
    }
    fill_color.set(ctx);
    ctx.fill_preserve()?;
    stroke_color.set(ctx);
    ctx.stroke()?;
    Ok(())
}

fn render_stuff(ctx: &Context, r: f64, tx: &Mat3) -> Result<(), Box<dyn Error>> {
    render_face(
        ctx,
        &[
            Vec3::new(0.0, 0.0, 0.0 + r / 2.0),
            Vec3::new(r, 0.0, 0.0 + r / 2.0),
            Vec3::new(r, r, 0.0 + r / 2.0),
            Vec3::new(0.0, r, 0.0 + r / 2.0),
        ],
        tx,
        &Color::from_rgba(0x00, 0xff, 0x00, 0.8),
        &Color::black(),
    )?;

    render_face(
        ctx,
        &[
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(r, 0.0, 0.0),
            Vec3::new(r, r, 0.0),
            Vec3::new(0.0, r, 0.0),
        ],
        tx,
        &Color::from_rgba(0xff, 0x00, 0x00, 0.8),
        &Color::black(),
    )?;

    // {
    //     let face = vec![
    //         Vec3::new(-r, -r, -r),
    //         Vec3::new(r, -r, -r),
    //         Vec3::new(r, r, -r),
    //         Vec3::new(-r, r, -r),
    //     ];

    //     ctx.save()?;
    //     Color::from_rgba(0xff, 0x00, 0x00, 0.8).set(ctx);
    //     render_path(ctx, &mut face.iter().map(|v| v.transform(&tx)))?;
    //     ctx.fill()?;
    //     ctx.restore()?;
    // }

    // {
    //     let face = vec![
    //         Vec3::new(-r, -r, r),
    //         Vec3::new(r, -r, r),
    //         Vec3::new(r, r, r),
    //         Vec3::new(-r, r, r),
    //     ];
    //     ctx.save()?;
    //     Color::from_rgba(0x00, 0xff, 0x00, 0.8).set(ctx);
    //     render_path(ctx, &mut face.iter().map(|v| v.transform(&tx)))?;
    //     ctx.fill()?;
    //     ctx.restore()?;
    // }
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
    Color::from_rgb(0x33, 0x33, 0x33).set(ctx);
    ctx.fill()?;
    ctx.restore()?;

    let origin = Point::new(width / 2.0, height * 0.9);
    ctx.translate(origin.x(), origin.y());
    ctx.save()?;
    ctx.new_path();
    ctx.move_to(-10.0, 0.0);
    ctx.line_to(10.0, 0.0);
    ctx.move_to(0.0, -10.0);
    ctx.line_to(0.0, 10.0);
    Color::white().set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    let tx = isometric_transform();
    let size = 40.0;
    let r = size / 2.0;

    ctx.save()?;
    for i in 0..10 {
        let i = (i * 100) as f64;
        let pa = Vec3::new(i, 0.0, 0.0).transform(&tx);
        let pb = Vec3::new(i, 0.0, 500.0).transform(&tx);
        ctx.move_to(pa.i, pa.j);
        ctx.line_to(pb.i, pb.j);
    }
    Color::from_rgba(0xff, 0xff, 0xff, 0.6).set(ctx);
    ctx.stroke()?;
    ctx.restore()?;

    render_stuff(ctx, r, &tx)?;

    Ok(())
}
