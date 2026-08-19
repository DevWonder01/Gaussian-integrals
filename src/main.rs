use macroquad::prelude::*;

// =========================================================================
// Camera
// =========================================================================

struct Camera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    center: Vec3,
}

impl Camera {
    fn new() -> Self {
        Self { yaw: -0.8, pitch: 0.6, distance: 5.5, center: vec3(0.0, 0.0, 0.0) }
    }
    fn pos(&self) -> Vec3 {
        let cp = self.pitch.cos();
        vec3(
            self.distance * cp * self.yaw.cos(),
            self.distance * cp * self.yaw.sin(),
            self.distance * self.pitch.sin(),
        ) + self.center
    }
    fn view_proj(&self, aspect: f32) -> Mat4 {
        let view = Mat4::look_at_rh(self.pos(), self.center, vec3(0.0, 0.0, 1.0));
        let proj = Mat4::perspective_rh(45f32.to_radians(), aspect, 0.1, 100.0);
        proj * view
    }
}

// =========================================================================
// Projection helpers
// =========================================================================

fn project(p: Vec3, vp: Mat4, ox: f32, oy: f32, w: f32, h: f32) -> Option<Vec2> {
    let c = vp * Vec4::new(p.x, p.y, p.z, 1.0);
    if c.w <= 0.0 { return None; }
    let ndc = vec3(c.x / c.w, c.y / c.w, c.z / c.w);
    Some(vec2(ox + (ndc.x + 1.0) * 0.5 * w, oy + (1.0 - ndc.y) * 0.5 * h))
}

fn draw_3d_line(a: Vec3, b: Vec3, thick: f32, col: Color, vp: Mat4, ox: f32, oy: f32, w: f32, h: f32) {
    if let (Some(sa), Some(sb)) = (project(a, vp, ox, oy, w, h), project(b, vp, ox, oy, w, h)) {
        draw_line(sa.x, sa.y, sb.x, sb.y, thick, col);
    }
}

fn draw_3d_text(txt: &str, p: Vec3, col: Color, sz: f32, vp: Mat4, ox: f32, oy: f32, w: f32, h: f32) {
    if let Some(s) = project(p, vp, ox, oy, w, h) {
        draw_text(txt, s.x, s.y, sz, col);
    }
}

// =========================================================================
// Gaussian math
// =========================================================================

/// f(x,y) = e^{-(x²+y²)}
fn gaussian_2d(x: f32, y: f32) -> f32 {
    (-x * x - y * y).exp()
}

/// Numerical integration of e^{-x²} from -limit to +limit using Simpson's rule
fn integrate_gaussian_1d(limit: f32, n: usize) -> f32 {
    let n = if n % 2 == 0 { n } else { n + 1 };
    let a = -limit;
    let b = limit;
    let h = (b - a) / n as f32;
    let mut sum = (-a * a).exp() + (-b * b).exp();
    for i in 1..n {
        let x = a + i as f32 * h;
        let w = if i % 2 == 0 { 2.0 } else { 4.0 };
        sum += w * (-x * x).exp();
    }
    sum * h / 3.0
}

/// Numerical double integral of e^{-(x²+y²)} over [-limit, limit]²
fn integrate_gaussian_2d(limit: f32, n: usize) -> f32 {
    let h = 2.0 * limit / n as f32;
    let mut sum = 0.0;
    for i in 0..n {
        for j in 0..n {
            let x = -limit + (i as f32 + 0.5) * h;
            let y = -limit + (j as f32 + 0.5) * h;
            sum += gaussian_2d(x, y) * h * h;
        }
    }
    sum
}

// =========================================================================
// Colormaps (matching scalar_fields aesthetics)
// =========================================================================

fn plasma(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if t < 0.25 {
        let f = t / 0.25;
        Color::new(0.05 + f * 0.30, 0.03, 0.30 + f * 0.25, 1.0)
    } else if t < 0.5 {
        let f = (t - 0.25) / 0.25;
        Color::new(0.35 + f * 0.30, 0.03 + f * 0.10, 0.55 - f * 0.05, 1.0)
    } else if t < 0.75 {
        let f = (t - 0.5) / 0.25;
        Color::new(0.65 + f * 0.25, 0.13 + f * 0.25, 0.50 - f * 0.20, 1.0)
    } else {
        let f = (t - 0.75) / 0.25;
        Color::new(0.90 + f * 0.08, 0.38 + f * 0.52, 0.30 - f * 0.18, 1.0)
    }
}

// =========================================================================
// Face for painter's algorithm
// =========================================================================

struct Face {
    v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3,
    depth: f32,
    height: f32,
}

// =========================================================================
// Main
// =========================================================================

#[macroquad::main("Gaussian Integral — I = √π")]
async fn main() {
    let mut cam = Camera::new();
    let mut prev_mouse = Vec2::ZERO;
    let mut egui_captured = false;

    // Surface parameters
    let mut grid_res: usize = 50;
    let mut domain = 3.0_f32;
    let mut opacity = 0.88_f32;
    let mut show_cross_x = true;
    let mut show_cross_y = true;
    let mut show_grid_lines = true;
    let mut integration_limit = 5.0_f32;
    let mut integration_n: usize = 200;

    // Derivation step expander state
    let mut show_derivation = true;

    let light_dir = vec3(1.0, 1.0, 2.0).normalize();

    loop {
        let sw = screen_width();
        let sh = screen_height();
        let mouse = vec2(mouse_position().0, mouse_position().1);

        // Camera controls (orbit + zoom) when egui doesn't capture
        if is_mouse_button_down(MouseButton::Left) && !egui_captured {
            let d = mouse - prev_mouse;
            cam.yaw += d.x * 0.007;
            cam.pitch = (cam.pitch - d.y * 0.007).clamp(-1.4, 1.4);
        }
        if !egui_captured {
            cam.distance = (cam.distance - mouse_wheel().1 * 0.35).clamp(2.0, 14.0);
        }
        prev_mouse = mouse;

        let vp = cam.view_proj(sw / sh);

        // Build faces
        let h = 2.0 * domain / grid_res as f32;
        let cam_pos = cam.pos();
        let mut faces: Vec<Face> = Vec::with_capacity(grid_res * grid_res);

        for i in 0..grid_res {
            for j in 0..grid_res {
                let x0 = -domain + i as f32 * h;
                let y0 = -domain + j as f32 * h;
                let x1 = x0 + h;
                let y1 = y0 + h;

                let p00 = vec3(x0, y0, gaussian_2d(x0, y0));
                let p10 = vec3(x1, y0, gaussian_2d(x1, y0));
                let p11 = vec3(x1, y1, gaussian_2d(x1, y1));
                let p01 = vec3(x0, y1, gaussian_2d(x0, y1));

                let center = (p00 + p10 + p11 + p01) * 0.25;
                let depth = (center - cam_pos).length();
                let height_val = center.z;

                faces.push(Face { v0: p00, v1: p10, v2: p11, v3: p01, depth, height: height_val });
            }
        }
        faces.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        // =====================================================================
        // Render
        // =====================================================================
        clear_background(Color::new(0.03, 0.04, 0.06, 1.0));

        // Ground grid
        let gz = -0.15;
        let gs = domain;
        for k in 0..=10 {
            let f = (k as f32 / 10.0) * 2.0 - 1.0;
            let c = f * gs;
            let gc = Color::new(0.18, 0.20, 0.26, 0.12);
            draw_3d_line(vec3(-gs, c, gz), vec3(gs, c, gz), 1.0, gc, vp, 0.0, 0.0, sw, sh);
            draw_3d_line(vec3(c, -gs, gz), vec3(c, gs, gz), 1.0, gc, vp, 0.0, 0.0, sw, sh);
        }

        // Axes
        let al = domain * 0.6;
        draw_3d_line(Vec3::ZERO, vec3(al, 0.0, 0.0), 2.0, Color::new(0.8, 0.2, 0.2, 0.5), vp, 0.0, 0.0, sw, sh);
        draw_3d_text("X", vec3(al + 0.1, 0.0, 0.0), Color::new(0.9, 0.4, 0.4, 0.8), 16.0, vp, 0.0, 0.0, sw, sh);
        draw_3d_line(Vec3::ZERO, vec3(0.0, al, 0.0), 2.0, Color::new(0.2, 0.8, 0.2, 0.5), vp, 0.0, 0.0, sw, sh);
        draw_3d_text("Y", vec3(0.0, al + 0.1, 0.0), Color::new(0.4, 0.9, 0.4, 0.8), 16.0, vp, 0.0, 0.0, sw, sh);
        draw_3d_line(Vec3::ZERO, vec3(0.0, 0.0, 1.2), 2.0, Color::new(0.2, 0.3, 0.9, 0.5), vp, 0.0, 0.0, sw, sh);
        draw_3d_text("Z", vec3(0.0, 0.0, 1.3), Color::new(0.4, 0.5, 0.9, 0.8), 16.0, vp, 0.0, 0.0, sw, sh);

        // Draw surface faces (painter's algorithm)
        for face in &faces {
            if let (Some(q0), Some(q1), Some(q2), Some(q3)) = (
                project(face.v0, vp, 0.0, 0.0, sw, sh),
                project(face.v1, vp, 0.0, 0.0, sw, sh),
                project(face.v2, vp, 0.0, 0.0, sw, sh),
                project(face.v3, vp, 0.0, 0.0, sw, sh),
            ) {
                let t_val = face.height.clamp(0.0, 1.0);
                let base = plasma(t_val);

                let normal = (face.v1 - face.v0).cross(face.v3 - face.v0).normalize();
                let lf = 0.35 + 0.65 * normal.dot(light_dir).abs();

                let col = Color::new(base.r * lf, base.g * lf, base.b * lf, opacity);
                draw_triangle(q0, q1, q2, col);
                draw_triangle(q0, q2, q3, col);

                if show_grid_lines {
                    let wc = Color::new(0.06, 0.06, 0.10, 0.12);
                    draw_line(q0.x, q0.y, q1.x, q1.y, 0.6, wc);
                    draw_line(q1.x, q1.y, q2.x, q2.y, 0.6, wc);
                    draw_line(q2.x, q2.y, q3.x, q3.y, 0.6, wc);
                    draw_line(q3.x, q3.y, q0.x, q0.y, 0.6, wc);
                }
            }
        }

        // Cross-section curves: e^{-x²} on xz-plane (y=0)
        if show_cross_x {
            let steps = 120;
            let cross_col = Color::new(1.0, 0.35, 0.2, 0.95);
            for k in 0..steps {
                let x0 = -domain + k as f32 * 2.0 * domain / steps as f32;
                let x1 = -domain + (k + 1) as f32 * 2.0 * domain / steps as f32;
                let z0 = (-x0 * x0).exp();
                let z1 = (-x1 * x1).exp();
                draw_3d_line(vec3(x0, 0.0, z0), vec3(x1, 0.0, z1), 3.0, cross_col, vp, 0.0, 0.0, sw, sh);
            }
            draw_3d_text("e^(-x²)", vec3(domain * 0.45, 0.0, (-domain * 0.45 * domain * 0.45_f32).exp() + 0.08), cross_col, 16.0, vp, 0.0, 0.0, sw, sh);
        }

        // Cross-section: e^{-y²} on yz-plane (x=0)
        if show_cross_y {
            let steps = 120;
            let cross_col = Color::new(0.2, 0.85, 0.4, 0.95);
            for k in 0..steps {
                let y0 = -domain + k as f32 * 2.0 * domain / steps as f32;
                let y1 = -domain + (k + 1) as f32 * 2.0 * domain / steps as f32;
                let z0 = (-y0 * y0).exp();
                let z1 = (-y1 * y1).exp();
                draw_3d_line(vec3(0.0, y0, z0), vec3(0.0, y1, z1), 3.0, cross_col, vp, 0.0, 0.0, sw, sh);
            }
            draw_3d_text("e^(-y²)", vec3(0.0, domain * 0.45, (-domain * 0.45 * domain * 0.45_f32).exp() + 0.08), cross_col, 16.0, vp, 0.0, 0.0, sw, sh);
        }

        // Peak label
        draw_3d_text("e^(-(x²+y²))", vec3(0.15, 0.15, 1.08), WHITE, 18.0, vp, 0.0, 0.0, sw, sh);

        // Title overlay
        draw_text("Gaussian Integral Visualization", 20.0, 30.0, 20.0, WHITE);
        draw_text(
            "Drag to rotate · Scroll to zoom",
            20.0, sh - 16.0, 13.0, Color::new(0.5, 0.55, 0.65, 0.7),
        );

        // =====================================================================
        // egui panel
        // =====================================================================
        let i_1d = integrate_gaussian_1d(integration_limit, integration_n);
        let i_2d = integrate_gaussian_2d(integration_limit, integration_n / 2);
        let pi_val = std::f32::consts::PI;
        let sqrt_pi = pi_val.sqrt();

        egui_macroquad::ui(|ctx| {
            egui::Window::new("Gaussian Integral")
                .default_width(360.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("I = ∫ e^(−x²) dx = √π");
                        ui.label(egui::RichText::new("The Gaussian Integral").weak());
                    });
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // ── Problem ──
                        ui.heading("Problem");
                        ui.colored_label(
                            egui::Color32::from_rgb(120, 200, 255),
                            "Evaluate   I = ∫_{−∞}^{∞} e^{−x²} dx",
                        );
                        ui.add_space(4.0);

                        // ── Derivation ──
                        ui.checkbox(&mut show_derivation, "Show Full Derivation");
                        if show_derivation {
                            ui.add_space(4.0);
                            ui.heading("Step 1 — Square the Integral");
                            ui.label("I² = (∫ e^{−x²} dx)(∫ e^{−y²} dy)");
                            ui.label("   = ∬ e^{−(x²+y²)} dx dy");
                            ui.colored_label(
                                egui::Color32::from_rgb(180, 150, 230),
                                "The 3D surface e^{−(x²+y²)} has radial symmetry.",
                            );

                            ui.add_space(6.0);
                            ui.heading("Step 2 — Polar Coordinates");
                            ui.label("x = r cos θ,   y = r sin θ");
                            ui.label("x² + y² = r²");
                            ui.label("Jacobian J = r");
                            ui.label("dx dy → r dr dθ");

                            ui.add_space(6.0);
                            ui.heading("Step 3 — Evaluate");
                            ui.label("I² = ∫₀²π ∫₀^∞ e^{−r²} r dr dθ");
                            ui.label("Substitution: u = r², du = 2r dr");
                            ui.label("I² = ∫₀²π ½ ∫₀^∞ e^{−u} du dθ");
                            ui.label("   = ∫₀²π ½ · 1 dθ = π");

                            ui.add_space(6.0);
                            ui.heading("Step 4 — Result");
                            ui.colored_label(
                                egui::Color32::from_rgb(100, 255, 150),
                                egui::RichText::new("I² = π  ⟹  I = √π ≈ 1.7724539")
                                    .strong().size(16.0),
                            );
                        }

                        ui.separator();

                        // ── Numerical verification ──
                        ui.heading("Numerical Verification");
                        ui.add(egui::Slider::new(&mut integration_limit, 2.0..=10.0).text("Integration limit"));
                        ui.add(egui::Slider::new(&mut integration_n, 50..=1000).text("Simpson points"));

                        ui.add_space(4.0);
                        ui.label(format!("∫ e^(−x²) dx ≈ {:.7}", i_1d));
                        ui.label(format!("Exact √π    = {:.7}", sqrt_pi));
                        ui.label(format!("Error        = {:.2e}", (i_1d - sqrt_pi).abs()));

                        ui.add_space(4.0);
                        ui.label(format!("∬ e^(−(x²+y²)) dA ≈ {:.6}", i_2d));
                        ui.label(format!("Exact π        = {:.6}", pi_val));
                        ui.label(format!("Error           = {:.2e}", (i_2d - pi_val).abs()));

                        ui.separator();

                        // ── Application ──
                        ui.heading("Application");
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 200, 100),
                            "The Gaussian integral normalizes the normal distribution:",
                        );
                        ui.label("p(x) = (1/σ√(2π)) e^{−(x−μ)²/(2σ²)}");
                        ui.label("∫ p(x) dx = 1  (total probability)");
                        ui.colored_label(
                            egui::Color32::from_rgb(180, 150, 230),
                            "Used for: measurement errors, noise, statistics.",
                        );

                        ui.separator();

                        // ── Visualization controls ──
                        ui.heading("Visualization");
                        ui.add(egui::Slider::new(&mut grid_res, 15..=80).text("Surface resolution"));
                        ui.add(egui::Slider::new(&mut domain, 1.5..=5.0).text("Domain extent"));
                        ui.add(egui::Slider::new(&mut opacity, 0.2..=1.0).text("Surface opacity"));
                        ui.checkbox(&mut show_cross_x, "Show e^(−x²) cross-section (red)");
                        ui.checkbox(&mut show_cross_y, "Show e^(−y²) cross-section (green)");
                        ui.checkbox(&mut show_grid_lines, "Show wireframe grid");
                        ui.add_space(10.0);
                    });
                });
            egui_captured = ctx.wants_pointer_input() || ctx.is_pointer_over_area();
        });

        egui_macroquad::draw();
        next_frame().await;
    }
}
