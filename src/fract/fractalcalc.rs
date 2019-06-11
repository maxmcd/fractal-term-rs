extern crate num;

use self::num::complex::{Complex, Complex64};

use leelib::matrix::Matrix;
use leelib::vector2::Vector2f;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

const DEFAULT_MANDELBROT_WIDTH: f64 = 4.0;
const DEFAULT_JULIA_WIDTH: f64 = 4.0;

/**
 *
 */
#[derive(Clone, Copy)]
pub enum FractalType {
    Mandelbrot,
    Julia(Complex64),
}

/**
 * Simple value object, passed around for use with FractalCalc methods
 */
#[derive(Clone, Copy)]
pub struct FractalSpecs {
    pub fractal_type: FractalType,
    pub max_val: u16,
    pub default_width: f64,
    pub default_center: Vector2f,
    pub element_ar: f64,
    pub num_threads: usize,
    pub use_multi_threads: bool,
}

impl FractalSpecs {
    pub fn new_mandelbrot_with_defaults(element_ar: f64) -> Self {
        FractalSpecs {
            fractal_type: FractalType::Mandelbrot,
            max_val: 500,
            default_width: DEFAULT_MANDELBROT_WIDTH,
            default_center: Vector2f::new(0.0, 0.0),
            element_ar,
            num_threads: 1,
            use_multi_threads: false,
        }
    }

    pub fn new_julia(c: Complex64, element_ar: f64) -> Self {
        FractalSpecs {
            fractal_type: FractalType::Julia(c),

            max_val: 500,
            default_width: DEFAULT_JULIA_WIDTH,
            default_center: Vector2f::new(0.0, 0.0),
            element_ar,
            num_threads: 1,
            use_multi_threads: false,
        }
    }
}

/**
 * 'Static' class
 * Fills in a `Matrix` with calculated fractal values
 */
pub struct FractalCalc;

impl FractalCalc {
    pub fn get_height(
        specs: &FractalSpecs,
        matrix_width: usize,
        full_matrix_height: usize,
        width: f64,
    ) -> f64 {
        let matrix_aspect_ratio = matrix_width as f64 / full_matrix_height as f64;
        width * (1.0 / matrix_aspect_ratio) * (1.0 / specs.element_ar)
    }

    pub fn write_matrix(
        specs: &FractalSpecs,
        center: Vector2f,
        width: f64,
        rotation: f64,
        matrix: &mut Matrix<u16>,
    ) {
        let h = matrix.height();
        FractalCalc::write_matrix_section(&specs, center, width, rotation, matrix, 0, h);
    }

    /**
     * Fills pre-existing 2d vector with mandelbrot set values
     *
     * width
     *      the width in 'mandelbrot space' which will be mapped to the width of the matrix
     * 		Note how height (in mandelbrot set's space) is derived from a combination of the
     * 		A/R of the full matrix height and element_aspect_ratio
     * center
     *      the center in 'mandelbrot space'
     * section
     *  	the matrix to be written to (which is a section of the full matrix)
     * full_matrix_offset
     *      the row from the full matrix where the section starts at
     * full_matrix_height
     *      height of the full matrix
     */
    pub fn write_matrix_section(
        specs: &FractalSpecs,
        center: Vector2f,
        width: f64,
        rotation: f64,
        section: &mut Matrix<u16>,
        full_matrix_offset: usize,
        full_matrix_height: usize,
    ) {
        let mandelbrot_height =
            FractalCalc::get_height(specs, section.width(), full_matrix_height, width);

        let element_w = width / section.width() as f64;
        let element_h = mandelbrot_height / full_matrix_height as f64;

        let slope_x = Vector2f::rotate(Vector2f::new(element_w, 0.0), rotation);
        let slope_y = Vector2f::rotate(Vector2f::new(0.0, element_h), rotation);

        let half_matrix_w = section.width() as f64 / 2.0;
        let half_matrix_h = full_matrix_height as f64 / 2.0;

        for index_y in 0..section.height() {
            let mut cursor = center;

            // move to left edge:
            let val = slope_x * -half_matrix_w;
            cursor = cursor + val;

            // move 'vertically' along 'left' edge:
            let val = slope_y * ((full_matrix_offset + index_y) as f64 - half_matrix_h);
            cursor = cursor + val;

            for index_x in 0..section.width() {
                let value = FractalCalc::get_value(&specs, cursor.x, cursor.y);
                section.set(index_x, index_y, value);

                // move 'right'
                cursor.x += slope_x.x;
                cursor.y += slope_x.y;
            }
        }
    }

    pub fn get_value(specs: &FractalSpecs, x: f64, y: f64) -> u16 {
        // ersatz-dynamic dispatch (tried other refactoring routes which didn't work out :( )
        match specs.fractal_type {
            FractalType::Mandelbrot => FractalCalc::get_mandelbrot_value(x, y, specs.max_val),
            FractalType::Julia(c) => FractalCalc::get_julia_value(&c, x, y, specs.max_val),
        }
    }

    fn get_mandelbrot_value(x: f64, y: f64, max_val: u16) -> u16 {
        let c = Complex { re: x, im: y };
        let mut z = Complex { re: 0f64, im: 0f64 };
        let mut val = 0;
        while z.norm_sqr().sqrt() < 2.0f64 && val < max_val {
            z = z * z + c;
            val += 1;
        }
        val
    }

    fn get_julia_value(c: &Complex64, x: f64, y: f64, max_val: u16) -> u16 {
        let mut z = Complex { re: x, im: y };
        for val in 0..max_val {
            let z_abs: f64 = (z.re * z.re + z.im * z.im).sqrt();
            if z_abs > 2.0 {
                return val;
            }
            z = z * z + c;
        }
        max_val
    }
}
