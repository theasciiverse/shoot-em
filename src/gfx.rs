//! A small software stand-in for the slice of OpenGL 1.1 the original
//! used: a matrix stack, glFrustum/gluLookAt, one positional light with
//! colour material, depth test, back-face culling, alpha blending,
//! triangles, quads, strips and lines, and the GLU sphere and cylinder.
//! Everything is drawn into an RGB float framebuffer that `ascii.rs`
//! turns into characters exactly the way the original turned its
//! glReadPixels buffer into console cells.

pub type V3 = [f32; 3];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4(pub [f32; 16]); // column major, like OpenGL

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);

    pub fn mul(&self, o: &Mat4) -> Mat4 {
        let a = &self.0;
        let b = &o.0;
        let mut r = [0.0; 16];
        for c in 0..4 {
            for row in 0..4 {
                let mut s = 0.0;
                for k in 0..4 {
                    s += a[k * 4 + row] * b[c * 4 + k];
                }
                r[c * 4 + row] = s;
            }
        }
        Mat4(r)
    }

    pub fn transform(&self, p: [f32; 4]) -> [f32; 4] {
        let m = &self.0;
        let mut r = [0.0; 4];
        for (row, out) in r.iter_mut().enumerate() {
            *out = m[row] * p[0] + m[4 + row] * p[1] + m[8 + row] * p[2] + m[12 + row] * p[3];
        }
        r
    }

    pub fn transform_dir(&self, v: V3) -> V3 {
        let m = &self.0;
        [
            m[0] * v[0] + m[4] * v[1] + m[8] * v[2],
            m[1] * v[0] + m[5] * v[1] + m[9] * v[2],
            m[2] * v[0] + m[6] * v[1] + m[10] * v[2],
        ]
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Mat4 {
        let mut m = Mat4::IDENTITY;
        m.0[12] = x;
        m.0[13] = y;
        m.0[14] = z;
        m
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Mat4 {
        let mut m = Mat4::IDENTITY;
        m.0[0] = x;
        m.0[5] = y;
        m.0[10] = z;
        m
    }

    /// glRotatef: angle in degrees about an axis.
    pub fn rotate(deg: f32, x: f32, y: f32, z: f32) -> Mat4 {
        let l = (x * x + y * y + z * z).sqrt();
        if l == 0.0 {
            return Mat4::IDENTITY;
        }
        let (x, y, z) = (x / l, y / l, z / l);
        let (s, c) = deg.to_radians().sin_cos();
        let t = 1.0 - c;
        Mat4([
            x * x * t + c,
            y * x * t + z * s,
            x * z * t - y * s,
            0.0,
            x * y * t - z * s,
            y * y * t + c,
            y * z * t + x * s,
            0.0,
            x * z * t + y * s,
            y * z * t - x * s,
            z * z * t + c,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    pub fn frustum(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4 {
        Mat4([
            2.0 * n / (r - l),
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 * n / (t - b),
            0.0,
            0.0,
            (r + l) / (r - l),
            (t + b) / (t - b),
            -(f + n) / (f - n),
            -1.0,
            0.0,
            0.0,
            -2.0 * f * n / (f - n),
            0.0,
        ])
    }

    pub fn look_at(eye: V3, center: V3, up: V3) -> Mat4 {
        let f = norm(sub(center, eye));
        let s = norm(cross(f, up));
        let u = cross(s, f);
        let m = Mat4([
            s[0], u[0], -f[0], 0.0, s[1], u[1], -f[1], 0.0, s[2], u[2], -f[2], 0.0, 0.0, 0.0, 0.0,
            1.0,
        ]);
        m.mul(&Mat4::translate(-eye[0], -eye[1], -eye[2]))
    }
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}
pub fn norm(a: V3) -> V3 {
    let l = len(a);
    if l == 0.0 {
        a
    } else {
        scale(a, 1.0 / l)
    }
}
pub fn dist(a: V3, b: V3) -> f32 {
    len(sub(a, b))
}

/// A vertex after the modelview transform, lit, before projection.
#[derive(Clone, Copy)]
struct EyeVert {
    p: V3,
    c: [f32; 4],
}

pub struct Renderer {
    pub w: usize,
    pub h: usize,
    pub color: Vec<[f32; 3]>,
    depth: Vec<f32>,
    pub proj: Mat4,
    pub mv: Mat4,
    stack: Vec<Mat4>,
    pub light_eye: [f32; 4],
    pub cur: [f32; 4],
    pub normal: V3,
    pub lighting: bool,
    pub blend: bool,
    pub cull: bool,
    near: f32,
}

impl Renderer {
    pub fn new(w: usize, h: usize) -> Self {
        Renderer {
            w,
            h,
            color: vec![[0.0; 3]; w * h],
            depth: vec![1.0; w * h],
            proj: Mat4::IDENTITY,
            mv: Mat4::IDENTITY,
            stack: Vec::new(),
            light_eye: [0.0, 0.0, 1.0, 0.0],
            cur: [1.0, 1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            lighting: true,
            blend: false,
            cull: false,
            near: 0.1,
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.w = w;
            self.h = h;
            self.color = vec![[0.0; 3]; w * h];
            self.depth = vec![1.0; w * h];
        }
    }

    /// The original's projection (FUN_00402688): a 45 degree glFrustum with
    /// near 0.1 and far 32, made as wide as the screen is.
    pub fn set_projection(&mut self, aspect: f32) {
        let top = 0.1 * 1.0 * 0.5; // near * tan(45 deg) * 0.5
        let right = top * aspect;
        self.near = 0.1;
        self.proj = Mat4::frustum(-right, right, -top, top, 0.1, 32.0);
    }

    pub fn clear(&mut self) {
        self.color.fill([0.0; 3]);
        self.depth.fill(1.0);
        self.mv = Mat4::IDENTITY;
        self.stack.clear();
    }

    pub fn load_identity(&mut self) {
        self.mv = Mat4::IDENTITY;
    }
    pub fn push(&mut self) {
        self.stack.push(self.mv);
    }
    pub fn pop(&mut self) {
        if let Some(m) = self.stack.pop() {
            self.mv = m;
        }
    }
    pub fn mult(&mut self, m: &Mat4) {
        self.mv = self.mv.mul(m);
    }
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.mult(&Mat4::translate(x, y, z));
    }
    pub fn rotate(&mut self, deg: f32, x: f32, y: f32, z: f32) {
        self.mult(&Mat4::rotate(deg, x, y, z));
    }
    pub fn look_at(&mut self, eye: V3, center: V3, up: V3) {
        self.mult(&Mat4::look_at(eye, center, up));
    }
    /// glLightfv(GL_LIGHT0, GL_POSITION, p): stored in eye space.
    pub fn light(&mut self, p: [f32; 4]) {
        self.light_eye = self.mv.transform(p);
    }
    pub fn color3(&mut self, r: f32, g: f32, b: f32) {
        self.cur = [r, g, b, 1.0];
    }
    pub fn color4(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.cur = [r, g, b, a];
    }

    /// Fixed-function lighting with GL_COLOR_MATERIAL: the global ambient
    /// of 0.2 plus the diffuse term of one white light.
    fn shade(&self, eye_p: V3, eye_n: V3) -> [f32; 4] {
        let c = self.cur;
        if !self.lighting {
            return c;
        }
        let l = if self.light_eye[3] == 0.0 {
            norm([self.light_eye[0], self.light_eye[1], self.light_eye[2]])
        } else {
            norm(sub(
                [self.light_eye[0], self.light_eye[1], self.light_eye[2]],
                eye_p,
            ))
        };
        let d = dot(norm(eye_n), l).max(0.0);
        let k = 0.2 + d;
        [
            (c[0] * k).min(1.0),
            (c[1] * k).min(1.0),
            (c[2] * k).min(1.0),
            c[3].clamp(0.0, 1.0),
        ]
    }

    fn eye_vert(&self, p: V3, n: V3) -> EyeVert {
        let e = self.mv.transform([p[0], p[1], p[2], 1.0]);
        let ep = [e[0], e[1], e[2]];
        let en = self.mv.transform_dir(n);
        EyeVert {
            p: ep,
            c: self.shade(ep, en),
        }
    }

    /// One triangle given in object space with per-vertex normals.
    pub fn tri(&mut self, v: [V3; 3], n: [V3; 3]) {
        let e = [
            self.eye_vert(v[0], n[0]),
            self.eye_vert(v[1], n[1]),
            self.eye_vert(v[2], n[2]),
        ];
        self.tri_eye(e);
    }

    /// A triangle with one face normal.
    pub fn tri_flat(&mut self, v: [V3; 3], n: V3) {
        self.tri(v, [n, n, n]);
    }

    pub fn quad(&mut self, v: [V3; 4], n: V3) {
        self.tri([v[0], v[1], v[2]], [n, n, n]);
        self.tri([v[0], v[2], v[3]], [n, n, n]);
    }

    fn tri_eye(&mut self, e: [EyeVert; 3]) {
        // clip against the near plane (z = -near in eye space)
        let zn = -self.near;
        let inside: Vec<bool> = e.iter().map(|v| v.p[2] <= zn).collect();
        let n_in = inside.iter().filter(|&&b| b).count();
        if n_in == 0 {
            return;
        }
        if n_in == 3 {
            self.raster(e[0], e[1], e[2]);
            return;
        }
        let mut poly: Vec<EyeVert> = Vec::with_capacity(4);
        for i in 0..3 {
            let a = e[i];
            let b = e[(i + 1) % 3];
            let ai = a.p[2] <= zn;
            let bi = b.p[2] <= zn;
            if ai {
                poly.push(a);
            }
            if ai != bi {
                let t = (zn - a.p[2]) / (b.p[2] - a.p[2]);
                poly.push(lerp_vert(a, b, t));
            }
        }
        for i in 1..poly.len().saturating_sub(1) {
            self.raster(poly[0], poly[i], poly[i + 1]);
        }
    }

    fn project(&self, v: &EyeVert) -> (f32, f32, f32) {
        let c = self.proj.transform([v.p[0], v.p[1], v.p[2], 1.0]);
        let iw = 1.0 / c[3];
        let x = (c[0] * iw * 0.5 + 0.5) * self.w as f32;
        let y = (0.5 - c[1] * iw * 0.5) * self.h as f32;
        let z = c[2] * iw * 0.5 + 0.5;
        (x, y, z)
    }

    fn raster(&mut self, a: EyeVert, b: EyeVert, c: EyeVert) {
        let pa = self.project(&a);
        let pb = self.project(&b);
        let pc = self.project(&c);
        let area = (pb.0 - pa.0) * (pc.1 - pa.1) - (pb.1 - pa.1) * (pc.0 - pa.0);
        if area == 0.0 {
            return;
        }
        // screen y points down, so counter-clockwise (front) faces have area < 0
        if self.cull && area > 0.0 {
            return;
        }
        let minx = pa.0.min(pb.0).min(pc.0).floor().max(0.0) as i32;
        let maxx = pa.0.max(pb.0).max(pc.0).ceil().min(self.w as f32 - 1.0) as i32;
        let miny = pa.1.min(pb.1).min(pc.1).floor().max(0.0) as i32;
        let maxy = pa.1.max(pb.1).max(pc.1).ceil().min(self.h as f32 - 1.0) as i32;
        if minx > maxx || miny > maxy {
            return;
        }
        let inv = 1.0 / area;
        for y in miny..=maxy {
            let fy = y as f32 + 0.5;
            for x in minx..=maxx {
                let fx = x as f32 + 0.5;
                let w0 = ((pb.0 - fx) * (pc.1 - fy) - (pb.1 - fy) * (pc.0 - fx)) * inv;
                let w1 = ((pc.0 - fx) * (pa.1 - fy) - (pc.1 - fy) * (pa.0 - fx)) * inv;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let z = w0 * pa.2 + w1 * pb.2 + w2 * pc.2;
                if !(0.0..=1.0).contains(&z) {
                    continue;
                }
                let i = y as usize * self.w + x as usize;
                if z > self.depth[i] {
                    continue;
                }
                let col = [
                    w0 * a.c[0] + w1 * b.c[0] + w2 * c.c[0],
                    w0 * a.c[1] + w1 * b.c[1] + w2 * c.c[1],
                    w0 * a.c[2] + w1 * b.c[2] + w2 * c.c[2],
                    w0 * a.c[3] + w1 * b.c[3] + w2 * c.c[3],
                ];
                self.plot(i, z, col);
            }
        }
    }

    fn plot(&mut self, i: usize, z: f32, col: [f32; 4]) {
        if self.blend {
            let a = col[3];
            let d = self.color[i];
            self.color[i] = [
                col[0] * a + d[0] * (1.0 - a),
                col[1] * a + d[1] * (1.0 - a),
                col[2] * a + d[2] * (1.0 - a),
            ];
        } else {
            self.color[i] = [col[0], col[1], col[2]];
        }
        self.depth[i] = z;
    }

    /// GL_LINE_LOOP through object-space points, lit with the current normal.
    pub fn line_loop(&mut self, pts: &[V3]) {
        let n = self.normal;
        let ev: Vec<EyeVert> = pts.iter().map(|&p| self.eye_vert(p, n)).collect();
        for i in 0..ev.len() {
            self.line_eye(ev[i], ev[(i + 1) % ev.len()]);
        }
    }

    fn line_eye(&mut self, mut a: EyeVert, mut b: EyeVert) {
        let zn = -self.near;
        if a.p[2] > zn && b.p[2] > zn {
            return;
        }
        if a.p[2] > zn {
            a = lerp_vert(a, b, (zn - a.p[2]) / (b.p[2] - a.p[2]));
        } else if b.p[2] > zn {
            b = lerp_vert(b, a, (zn - b.p[2]) / (a.p[2] - b.p[2]));
        }
        let pa = self.project(&a);
        let pb = self.project(&b);
        let steps = (pb.0 - pa.0).abs().max((pb.1 - pa.1).abs()).ceil().max(1.0) as i32;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let x = pa.0 + (pb.0 - pa.0) * t;
            let y = pa.1 + (pb.1 - pa.1) * t;
            if x < 0.0 || y < 0.0 || x >= self.w as f32 || y >= self.h as f32 {
                continue;
            }
            let z = pa.2 + (pb.2 - pa.2) * t;
            let i = y as usize * self.w + x as usize;
            if z <= self.depth[i] {
                let c = [
                    a.c[0] + (b.c[0] - a.c[0]) * t,
                    a.c[1] + (b.c[1] - a.c[1]) * t,
                    a.c[2] + (b.c[2] - a.c[2]) * t,
                    1.0,
                ];
                self.plot(i, z, c);
            }
        }
    }
}

fn lerp_vert(a: EyeVert, b: EyeVert, t: f32) -> EyeVert {
    EyeVert {
        p: [
            a.p[0] + (b.p[0] - a.p[0]) * t,
            a.p[1] + (b.p[1] - a.p[1]) * t,
            a.p[2] + (b.p[2] - a.p[2]) * t,
        ],
        c: [
            a.c[0] + (b.c[0] - a.c[0]) * t,
            a.c[1] + (b.c[1] - a.c[1]) * t,
            a.c[2] + (b.c[2] - a.c[2]) * t,
            a.c[3] + (b.c[3] - a.c[3]) * t,
        ],
    }
}

/// gluSphere: `slices` around the z axis, `stacks` from +z to -z, smooth
/// normals, or one normal per face for GLU_FLAT.
pub fn sphere(r: &mut Renderer, radius: f32, slices: usize, stacks: usize, flat: bool) {
    let pt = |i: usize, j: usize| -> V3 {
        let theta = std::f32::consts::PI * j as f32 / stacks as f32;
        let phi = 2.0 * std::f32::consts::PI * i as f32 / slices as f32;
        [
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        ]
    };
    for j in 0..stacks {
        for i in 0..slices {
            let a = pt(i, j);
            let b = pt(i, j + 1);
            let c = pt(i + 1, j + 1);
            let d = pt(i + 1, j);
            let s = |v: V3| scale(v, radius);
            // counter-clockwise seen from outside
            let tris = [[a, b, c], [a, c, d]];
            for t in tris {
                if len(sub(t[0], t[1])) < 1e-6
                    || len(sub(t[1], t[2])) < 1e-6
                    || len(sub(t[0], t[2])) < 1e-6
                {
                    continue;
                }
                if flat {
                    let n = norm(cross(sub(t[1], t[0]), sub(t[2], t[0])));
                    r.tri([s(t[0]), s(t[1]), s(t[2])], [n, n, n]);
                } else {
                    r.tri([s(t[0]), s(t[1]), s(t[2])], [t[0], t[1], t[2]]);
                }
            }
        }
    }
}

/// gluCylinder along +z, no caps, normals outward.
pub fn cylinder(r: &mut Renderer, base: f32, top: f32, height: f32, slices: usize) {
    for i in 0..slices {
        let p0 = 2.0 * std::f32::consts::PI * i as f32 / slices as f32;
        let p1 = 2.0 * std::f32::consts::PI * (i + 1) as f32 / slices as f32;
        let n0 = [p0.cos(), p0.sin(), 0.0];
        let n1 = [p1.cos(), p1.sin(), 0.0];
        let a = [base * p0.cos(), base * p0.sin(), 0.0];
        let b = [base * p1.cos(), base * p1.sin(), 0.0];
        let c = [top * p1.cos(), top * p1.sin(), height];
        let d = [top * p0.cos(), top * p0.sin(), height];
        r.tri([a, b, c], [n0, n1, n1]);
        r.tri([a, c, d], [n0, n1, n0]);
    }
}

/// FUN_0040a48c: an axis-aligned box with half extents (x, y, z).
pub fn boxx(r: &mut Renderer, x: f32, y: f32, z: f32) {
    let (nx, ny, nz) = (-x, -y, -z);
    r.quad(
        [[nx, ny, nz], [nx, y, nz], [x, y, nz], [x, ny, nz]],
        [0.0, 0.0, -1.0],
    );
    r.quad(
        [[nx, y, z], [nx, ny, z], [x, ny, z], [x, y, z]],
        [0.0, 0.0, 1.0],
    );
    r.quad(
        [[nx, ny, nz], [nx, ny, z], [nx, y, z], [nx, y, nz]],
        [-1.0, 0.0, 0.0],
    );
    r.quad(
        [[x, ny, z], [x, ny, nz], [x, y, nz], [x, y, z]],
        [1.0, 0.0, 0.0],
    );
    r.quad(
        [[nx, ny, z], [nx, ny, nz], [x, ny, nz], [x, ny, z]],
        [0.0, -1.0, 0.0],
    );
    r.quad(
        [[nx, y, nz], [nx, y, z], [x, y, z], [x, y, nz]],
        [0.0, 1.0, 0.0],
    );
}

/// OpenGL-style primitives: (mode, [(vertex, normal)]).
pub type Prims<'a> = &'a [(u8, &'a [(V3, V3)])];

/// Draws a primitive list (the extracted intro logo).
pub fn prims(r: &mut Renderer, list: Prims) {
    for (mode, verts) in list {
        match mode {
            4 => {
                for t in verts.chunks(3) {
                    if t.len() == 3 {
                        r.tri([t[0].0, t[1].0, t[2].0], [t[0].1, t[1].1, t[2].1]);
                    }
                }
            }
            5 => {
                for i in 2..verts.len() {
                    let (a, b, c) = if i % 2 == 0 {
                        (verts[i - 2], verts[i - 1], verts[i])
                    } else {
                        (verts[i - 1], verts[i - 2], verts[i])
                    };
                    r.tri([a.0, b.0, c.0], [a.1, b.1, c.1]);
                }
            }
            7 => {
                for q in verts.chunks(4) {
                    if q.len() == 4 {
                        r.tri([q[0].0, q[1].0, q[2].0], [q[0].1, q[1].1, q[2].1]);
                        r.tri([q[0].0, q[2].0, q[3].0], [q[0].1, q[2].1, q[3].1]);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lit_sphere_fills_the_middle_of_the_screen() {
        let mut r = Renderer::new(40, 40);
        r.set_projection(1.0);
        r.clear();
        r.light([0.0, 0.0, 10.0, 1.0]);
        r.translate(0.0, 0.0, -3.0);
        r.color3(1.0, 0.0, 0.0);
        r.cull = true;
        sphere(&mut r, 1.0, 8, 8, false);
        let mid = r.color[20 * 40 + 20];
        assert!(mid[0] > 0.8 && mid[1] == 0.0, "{mid:?}");
        assert_eq!(r.color[0], [0.0; 3]);
    }

    #[test]
    fn rotation_matches_opengl() {
        let m = Mat4::rotate(90.0, 0.0, 1.0, 0.0);
        let v = m.transform([1.0, 0.0, 0.0, 1.0]);
        assert!((v[2] + 1.0).abs() < 1e-5, "{v:?}");
    }
}
