---

# Monte Carlo Polygon Intersection Simulator

A **high-performance GPU-accelerated tool** to compute the probability of random line intersections within regular polygons.

---

## Overview

This project uses **Monte Carlo simulation** to estimate the probability that two randomly selected line segments within a regular polygon will intersect. The simulation is accelerated using **GPU computing** via the **WGPU API**, allowing it to efficiently calculate results for polygons with varying numbers of sides (from 3 to 500).

---

## Features

- **GPU-accelerated computation** using WGPU and compute shaders
- High precision with **1 billion iterations per polygon**
- Automatic **Excel output** with probabilities and confidence intervals
- Regular polygon generation with configurable number of sides
- Statistical analysis, including **standard deviation** and **95% confidence intervals**

---

## Key Findings

The project demonstrates that as the number of sides of a regular polygon increases, the probability of line segment intersection approaches **1/3**. The relationship follows the equation:

$$
y = \frac{1}{3} + \frac{1}{3 - 3n} + \frac{1}{n}
$$

Where `n` is the number of sides of the polygon and `y` is the probability of intersection.

---

## Requirements

- **Rust** (2021 edition)
- **GPU** with compute shader support
- Compatible OS: **Windows**, **Linux**, or **macOS**

---

## Dependencies

- Rust dependencies and GPU drivers as specified in `Cargo.toml`

---

## Building and Running

1. **Clone the repository**:
   ```bash
   git clone https://github.com/sudhakara-ambati/ngon-theorem.git
   cd ngon-theorem
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run the simulation**:
   ```bash
   cargo run --release
   ```

4. The program will generate a file named `polygon_intersection_results_gpu.xlsx` containing the simulation results.

---

## How It Works

For each number of sides (`n = 3 to 500`):

1. **Generate** a regular polygon with `n` sides.
2. **Run** 1 billion Monte Carlo iterations on the GPU.
3. For each iteration:
   - Select two random line segments within the polygon.
   - Check if they intersect within the polygon's interior.
4. **Calculate** the probability as:
   \[
   \text{Probability} = \frac{\text{Number of Intersections}}{\text{Total Iterations}}
   \]
5. **Record** the result with confidence intervals.

---

## GPU Acceleration

The GPU acceleration is achieved through **compute shaders** written in **WGSL** that:

- Generate random points on polygon edges
- Detect line intersections
- Test if intersection points lie within the polygon
- Aggregate results across thousands of parallel threads

The simulation distributes work across **4096 workgroups** with **256 threads each**, for a total of **1,048,576 parallel threads**. Each thread processes a portion of the total iterations.

---

## Implementation Details

The project consists of two main components:

1. **Rust Host Code** (`main.rs`):
   - Handles GPU setup, memory management, and result processing.

2. **WGSL Shader Code** (`monte_carlo.wgsl`):
   - Performs the Monte Carlo simulation on the GPU.

---
