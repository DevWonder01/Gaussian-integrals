# Interactive Gaussian Integral Simulation (I = √π)

An interactive 3D visualization and analytical tool built in **Rust** using **Macroquad** and **egui**. This application provides a real-time 3D simulation of the Gaussian surface $f(x, y) = e^{-(x^2+y^2)}$, its orthogonal cross-sections $e^{-x^2}$ and $e^{-y^2}$, and step-by-step mathematical proof of the classic integral:

$$I = \int_{-\infty}^{\infty} e^{-x^2} \, dx = \sqrt{\pi}$$

---

## Key Features

* **Real-Time 3D Rendering**: Custom software-rasterized 3D viewport utilizing painter's back-to-front depth sorting, dynamic Lambertian lighting, and Plasma height colormapping.
* **Orthogonal Cross-Section Highlights**: High-contrast rendering of the 2D Gaussian curves $e^{-x^2}$ (along the XZ-plane) and $e^{-y^2}$ (along the YZ-plane).
* **Interactive Mathematical Proof**: Step-by-step breakdown embedded directly within an interactive GUI panel, detailing:
  1. Integral squaring trick ($I^2 = \iint_{\mathbb{R}^2} e^{-(x^2+y^2)} \, dx \, dy$)
  2. Polar coordinate transformation ($x = r\cos\theta, y = r\sin\theta$)
  3. Jacobian determinant derivation ($J = r$)
  4. Integration via $u$-substitution ($u = r^2, du = 2r\,dr \implies I^2 = \pi$)
* **Numerical Verification Engine**: Live 1D Simpson's quadrature and 2D midpoint Riemann summation to visually confirm convergence toward $\sqrt{\pi} \approx 1.7724539$ and $\pi \approx 3.1415927$.
* **Physics & Statistical Applications**: Real-time overview of how $I = \sqrt{\pi}$ normalizes standard normal distributions and Gaussian wave packets in quantum mechanics.

---

## Mathematical Overview

The standard Gaussian integral cannot be evaluated in closed form using elementary antiderivatives. The canonical solution squares the integral to transform it into a double integral over the Cartesian plane:

$$I^2 = \left( \int_{-\infty}^{\infty} e^{-x^2} \, dx \right) \left( \int_{-\infty}^{\infty} e^{-y^2} \, dy \right) = \iint_{\mathbb{R}^2} e^{-(x^2+y^2)} \, dx \, dy$$

Transforming to polar coordinates $(r, \theta)$:

$$x = r\cos\theta, \quad y = r\sin\theta, \quad x^2 + y^2 = r^2$$

The area element transforms using the Jacobian determinant:

$$J = \det \begin{bmatrix} \frac{\partial x}{\partial r} & \frac{\partial x}{\partial \theta} \\ \frac{\partial y}{\partial r} & \frac{\partial y}{\partial \theta} \end{bmatrix} = \det \begin{bmatrix} \cos\theta & -r\sin\theta \\ \sin\theta & r\cos\theta \end{bmatrix} = r$$

Substituting into the integral:

$$I^2 = \int_{0}^{2\pi} \int_{0}^{\infty} e^{-r^2} r \, dr \, d\theta$$

Using $u = r^2 \implies du = 2r\,dr$:

$$I^2 = \int_{0}^{2\pi} \left[ \frac{1}{2} \int_{0}^{\infty} e^{-u} \, du \right] d\theta = \frac{1}{2} \int_{0}^{2\pi} 1 \, d\theta = \pi$$

Taking the positive square root yields the exact result:

$$I = \sqrt{\pi}$$

---

## Technology Stack

* **Language**: Rust (2021 Edition)
* **Graphics Engine**: [Macroquad 0.4](https://github.com/not-flodd/macroquad) — Lightweight 2D/3D cross-platform engine
* **GUI Framework**: [egui 0.31](https://github.com/emilk/egui) / `egui-macroquad 0.17` — Immediate mode floating interface
* **Math / Geometry**: Native SIMD-accelerated linear algebra operations (`glam` / `macroquad::math`)

---

## Getting Started

### Prerequisites

Ensure you have the Rust toolchain installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Linux (Ubuntu/Debian), install the standard X11 and OpenGL dependencies:

```bash
sudo apt update
sudo apt install pkg-config libx11-dev libasound2-dev libgl1-mesa-dev
```

### Build & Run

Clone the repository and run in release mode for optimal frame rates:

```bash
git clone https://github.com/Simulations/Gaussian-integral.git
cd Gaussian-integral

# Run debug build
cargo run

# Run optimized release build
cargo run --release
```

---

## Controls & Interactivity

| Action | Control |
| :--- | :--- |
| **Orbit Camera** | Click & drag left mouse button on the 3D viewport |
| **Zoom In / Out** | Mouse scroll wheel |
| **Derivation Steps** | Toggle "Show Full Derivation" in the GUI panel |
| **Surface Resolution** | Adjust grid slider ($15 \times 15$ to $80 \times 80$) |
| **Domain Extent** | Adjust domain slider ($[-1.5, 1.5]$ to $[-5.0, 5.0]$) |
| **Cross-Sections** | Toggle $e^{-x^2}$ and $e^{-y^2}$ boundary curves |

---

## License

Distributed under the MIT License. See `LICENSE` for details.
