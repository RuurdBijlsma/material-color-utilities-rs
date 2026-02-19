/*
 * Copyright 2025 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/// Utility methods for mathematical operations.
pub struct MathUtils;

impl MathUtils {
    /// The linear interpolation function.
    ///
    /// # Returns
    /// `start` if `amount` = 0 and `stop` if `amount` = 1
    pub fn lerp(start: f64, stop: f64, amount: f64) -> f64 {
        (1.0 - amount) * start + amount * stop
    }

    /// Sanitizes a degree measure as an integer.
    ///
    /// # Returns
    /// A degree measure between 0 (inclusive) and 360 (exclusive).
    pub fn sanitize_degrees_int(degrees: i32) -> i32 {
        let mut degrees = degrees % 360;
        if degrees < 0 {
            degrees += 360;
        }
        degrees
    }

    /// Sanitizes a degree measure as a floating-point number.
    ///
    /// # Returns
    /// A degree measure between 0.0 (inclusive) and 360.0 (exclusive).
    pub fn sanitize_degrees_double(degrees: f64) -> f64 {
        let mut degrees = degrees % 360.0;
        if degrees < 0.0 {
            degrees += 360.0;
        }
        degrees
    }

    /// Sign of direction change needed to travel from one angle to another.
    ///
    /// For angles that are 180 degrees apart from each other, both directions have the same travel
    /// distance, so either direction is shortest. The value 1.0 is returned in this case.
    ///
    /// # Arguments
    /// * `from` - The angle travel starts from, in degrees.
    /// * `to` - The angle travel ends at, in degrees.
    ///
    /// # Returns
    /// -1.0 if decreasing `from` leads to the shortest travel distance, 1.0 if increasing `from` leads
    /// to the shortest travel distance.
    pub fn rotation_direction(from: f64, to: f64) -> f64 {
        let increasing_difference = Self::sanitize_degrees_double(to - from);
        if increasing_difference <= 180.0 {
            1.0
        } else {
            -1.0
        }
    }

    /// Distance of two points on a circle, represented using degrees.
    pub fn difference_degrees(a: f64, b: f64) -> f64 {
        180.0 - ((a - b).abs() - 180.0).abs()
    }

    /// Multiplies a 1x3 row vector with a 3x3 matrix.
    pub fn matrix_multiply(row: [f64; 3], matrix: [[f64; 3]; 3]) -> [f64; 3] {
        let a = row[0] * matrix[0][0] + row[1] * matrix[0][1] + row[2] * matrix[0][2];
        let b = row[0] * matrix[1][0] + row[1] * matrix[1][1] + row[2] * matrix[1][2];
        let c = row[0] * matrix[2][0] + row[1] * matrix[2][1] + row[2] * matrix[2][2];
        [a, b, c]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(MathUtils::lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(MathUtils::lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(MathUtils::lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(MathUtils::lerp(10.0, 20.0, 0.25), 12.5);
    }

    #[test]
    fn test_sanitize_degrees_int() {
        assert_eq!(MathUtils::sanitize_degrees_int(0), 0);
        assert_eq!(MathUtils::sanitize_degrees_int(360), 0);
        assert_eq!(MathUtils::sanitize_degrees_int(720), 0);
        assert_eq!(MathUtils::sanitize_degrees_int(180), 180);
        assert_eq!(MathUtils::sanitize_degrees_int(-90), 270);
        assert_eq!(MathUtils::sanitize_degrees_int(-450), 270);
    }

    #[test]
    fn test_sanitize_degrees_double() {
        assert_eq!(MathUtils::sanitize_degrees_double(0.0), 0.0);
        assert_eq!(MathUtils::sanitize_degrees_double(360.0), 0.0);
        assert_eq!(MathUtils::sanitize_degrees_double(180.0), 180.0);
        assert_eq!(MathUtils::sanitize_degrees_double(-90.0), 270.0);
        assert_eq!(MathUtils::sanitize_degrees_double(-450.0), 270.0);
    }

    #[test]
    fn test_rotation_direction() {
        assert_eq!(MathUtils::rotation_direction(0.0, 10.0), 1.0);
        assert_eq!(MathUtils::rotation_direction(10.0, 0.0), -1.0);
        assert_eq!(MathUtils::rotation_direction(0.0, 180.0), 1.0);
        assert_eq!(MathUtils::rotation_direction(0.0, 190.0), -1.0);
        assert_eq!(MathUtils::rotation_direction(190.0, 0.0), 1.0);
    }

    #[test]
    fn test_difference_degrees() {
        assert_eq!(MathUtils::difference_degrees(0.0, 10.0), 10.0);
        assert_eq!(MathUtils::difference_degrees(10.0, 0.0), 10.0);
        assert_eq!(MathUtils::difference_degrees(0.0, 180.0), 180.0);
        assert_eq!(MathUtils::difference_degrees(0.0, 190.0), 170.0);
        assert_eq!(MathUtils::difference_degrees(190.0, 0.0), 170.0);
        assert_eq!(MathUtils::difference_degrees(350.0, 10.0), 20.0);
    }

    #[test]
    fn test_matrix_multiply() {
        let row = [1.0, 2.0, 3.0];
        let matrix = [
            [1.0, 0.0, 0.0],
            [0.1, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let result = MathUtils::matrix_multiply(row, matrix);
        // a = 1*1 + 2*0 + 3*0 = 1
        // b = 1*0.1 + 2*1 + 3*0 = 2.1
        // c = 1*0 + 2*0 + 3*1 = 3
        assert_eq!(result, [1.0, 2.1, 3.0]);
    }
}
