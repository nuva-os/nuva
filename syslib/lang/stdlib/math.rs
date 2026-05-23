/*
 * Nuva OS - SystemLibrary - Lang
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.

 */


/// MathematicsConstant
pub mod constants {
 /// Pi
 pub const PI: f64 = 3.14159265358979323846;
 /// selfthenLogarithm base
 pub const E: f64 = 2.71828182845904523536;
 /// 2 selfthenLogarithm
 pub const LN2: f64 = 0.693147180559945309417;
 /// 10 selfthenLogarithm
 pub const LN10: f64 = 2.30258509299404568402;
 /// 2 flatmethodRoot
 pub const SQRT2: f64 = 1.41421356237309504880;
 /// 1/2 flatmethodRoot
 pub const SQRT1_2: f64 = 0.70710678118654752440;
}

/// Absolute Value
pub fn abs(x: f64) -> f64 {
 if x < 0.0 { -x } else { x }
}

/// flatmethodRoot
pub fn sqrt(x: f64) -> f64 {
 if x < 0.0 {
 return f64::NAN;
 }
 
 // Iterationlaw
 let mut guess = x / 2.0;
 for _ in 0..20 {
 guess = (guess + x / guess) / 2.0;
 }
 guess
}

/// Poweroperationcalculation
pub fn pow(base: f64, exp: f64) -> f64 {
 // use exp(exp * ln(base))
 exp(exp * ln(base))
}

/// selfthenLogarithm
pub fn ln(x: f64) -> f64 {
 if x <= 0.0 {
 return f64::NEG_INFINITY;
 }
 
 // Taylor Expansion
 let mut result = 0.0;
 let mut term = (x - 1.0) / (x + 1.0);
 let term_sq = term * term;
 
 for i in 0..100 {
 result += term / (2.0 * i as f64 + 1.0);
 term *= term_sq;
 }
 
 result * 2.0
}

/// ExponentialFunction
pub fn exp(x: f64) -> f64 {
 // Taylor Expansion
 let mut result = 1.0;
 let mut term = 1.0;
 
 for i in 1..100 {
 term *= x / i as f64;
 result += term;
 }
 
 result
}

/// Sine
pub fn sin(x: f64) -> f64 {
 // Normalizationto [-π, π]
 let x = x % (2.0 * constants::PI);
 let x = if x > constants::PI { x - 2.0 * constants::PI } else if x < -constants::PI { x + 2.0 * constants::PI } else { x };
 
 // Taylor Expansion
 let mut result = 0.0;
 let mut term = x;
 let x_sq = x * x;
 
 for i in 0..20 {
 result += term;
 term *= -x_sq / ((2.0 * i as f64 + 2.0) * (2.0 * i as f64 + 3.0));
 }
 
 result
}

/// Cosine
pub fn cos(x: f64) -> f64 {
 sin(x + constants::PI / 2.0)
}

/// Tangent
pub fn tan(x: f64) -> f64 {
 sin(x) / cos(x)
}

/// Arcsine
pub fn asin(x: f64) -> f64 {
 if x < -1.0 || x > 1.0 {
 return f64::NAN;
 }
 
 // Taylor Expansion
 let mut result = 0.0;
 let mut term = x;
 let x_sq = x * x;
 
 for i in 0..20 {
 result += term / (2.0 * i as f64 + 1.0);
 term *= x_sq * (2.0 * i as f64 + 1.0) * (2.0 * i as f64 + 1.0) / ((2.0 * i as f64 + 2.0) * (2.0 * i as f64 + 3.0));
 }
 
 result
}

/// Arccosine
pub fn acos(x: f64) -> f64 {
 constants::PI / 2.0 - asin(x)
}

/// Arctangent
pub fn atan(x: f64) -> f64 {
 // Taylor Expansion
 let mut result = 0.0;
 let mut term = x;
 let x_sq = x * x;
 
 for i in 0..20 {
 result += term / (2.0 * i as f64 + 1.0);
 term *= -x_sq;
 }
 
 result
}

/// doubleParameterArctangent
pub fn atan2(y: f64, x: f64) -> f64 {
 if x > 0.0 {
 atan(y / x)
 } else if x < 0.0 {
 if y >= 0.0 {
 atan(y / x) + constants::PI
 } else {
 atan(y / x) - constants::PI
 }
 } else {
 if y > 0.0 {
 constants::PI / 2.0
 } else if y < 0.0 {
 -constants::PI / 2.0
 } else {
 0.0
 }
 }
}

/// directiondownloadFloor
pub fn floor(x: f64) -> f64 {
 let i = x as i64 as f64;
 if x < i { i - 1.0 } else { i }
}

/// directionuploadFloor
pub fn ceil(x: f64) -> f64 {
 let i = x as i64 as f64;
 if x > i { i + 1.0 } else { i }
}

/// Round
pub fn round(x: f64) -> f64 {
 if x >= 0.0 {
 floor(x + 0.5)
 } else {
 ceil(x - 0.5)
 }
}

/// Modulo
pub fn fmod(x: f64, y: f64) -> f64 {
 x - (x / y).floor() * y
}

/// Maxvalue
pub fn max(x: f64, y: f64) -> f64 {
 if x > y { x } else { y }
}

/// Minvalue
pub fn min(x: f64, y: f64) -> f64 {
 if x < y { x } else { y }
}

/// SignFunction
pub fn signum(x: f64) -> f64 {
 if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
}