use crate::primes::{factorial, PrimeFactorization};
use crate::rational::Rational;

/// Compute the Wigner 3j coefficient for the given `j1`, `j2`, `j2`, `m1`,
/// `m2`, `m3`.
#[no_mangle]
pub extern fn wigner_3j(j1: u32, j2: u32, j3: u32, m1: i32, m2: i32, m3: i32) -> f64 {
    if m1.abs() as u32 > j1 {
        panic!("invalid j/m in wigner3j: {}/{}", j1, m1);
    } else if m2.abs() as u32 > j2 {
        panic!("invalid j/m in wigner3j: {}/{}", j2, m2);
    } else if m3.abs() as u32 > j3 {
        panic!("invalid j/m in wigner3j: {}/{}", j3, m3);
    }

    if !triangle_condition(j1, j2, j3) || m1 + m2 + m3 != 0 {
        return 0.0;
    }

    let (j1, j2, j3, m1, m2, _, mut sign) = reorder3j(j1, j2, j3, m1, m2, m3, 1);

    let alpha1 = j2 as i32 - m1 - j3 as i32;
    let alpha2 = j1 as i32 + m2 - j3 as i32;
    let beta1 = (j1 + j2 - j3) as i32;
    let beta2 = j1 as i32 - m1;
    let beta3 = j2 as i32 + m2;

    // extra sign in definition: alpha1 - alpha2 = j1 + m2 - j2 + m1 = j1 - j2 + m3
    if (alpha1 - alpha2) % 2 != 0 {
        sign *= -1;
    }

    let s1 = triangle_coefficient(j1, j2, j3);

    debug_assert!(beta2 >= 0);
    let mut s2 = factorial(beta2 as u32);

    debug_assert!((beta1 - alpha1) >= 0);
    s2 *= factorial((beta1 - alpha1) as u32);

    debug_assert!((beta1 - alpha2) >= 0);
    s2 *= factorial((beta1 - alpha2) as u32);

    debug_assert!(beta3 >= 0);
    s2 *= factorial(beta3 as u32);

    debug_assert!((beta3 - alpha1) >= 0);
    s2 *= factorial((beta3 - alpha1) as u32);

    debug_assert!((beta2 - alpha2) >= 0);
    s2 *= factorial((beta2 - alpha2) as u32);

    let (series_numerator, series_denominator) = compute_3j_series(beta1, beta2, beta3, alpha1, alpha2);

    let mut numerator = s1.numerator * s2;
    numerator.sign *= sign;
    let mut s = Rational::new(numerator, s1.denominator);

    let series_denominator = Rational::new(PrimeFactorization::one(), series_denominator);

    // insert series denominator in the root, this improves precision compared
    // to immediately converting the full series to f64
    s *= &series_denominator;
    s *= &series_denominator;
    s.simplify();

    return series_numerator * s.signed_root();
}

/// Compute the Clebsch-Gordan coefficient <j1 m1 ; j2 m2 | j3 m3> using their
/// relation to Wigner 3j coefficients:
///
/// ```text
/// <j1 m1 ; j2 m2 | j3 m3> = (-1)^(j1 - j2 + m3) sqrt(2*j3 + 1) wigner_3j(j1, j2, j3, m1, m2, -m3)
/// ```
#[no_mangle]
pub extern fn clebsch_gordan(j1: u32, m1: i32, j2: u32, m2: i32, j3: u32, m3: i32) -> f64 {
    let mut w3j = wigner_3j(j1, j2, j3, m1, m2, -m3);

    w3j *= f64::sqrt((2 * j3 + 1) as f64);
    if (j1 as i32 - j2 as i32 + m3) % 2 != 0 {
        return -w3j;
    } else {
        return w3j;
    }
}

/// check the triangle condition on j1, j2, j3, i.e. `|j1 - j2| <= j3 <= j1 + j2`
fn triangle_condition(j1: u32, j2: u32, j3: u32) -> bool {
    return (j3 <= j1 + j2) && (j1 <= j2 + j3) && (j2 <= j3 + j1);
}

// reorder j1/m1, j2/m2, j3/m3 such that j1 >= j2 >= j3 and m1 >= 0 or m1 == 0 && m2 >= 0
fn reorder3j(j1: u32, j2: u32, j3: u32, m1: i32, m2: i32, m3: i32, mut sign: i8) -> (u32, u32, u32, i32, i32, i32, i8) {
    if j1 < j2 {
        return reorder3j(j2, j1, j3, m2, m1, m3, -sign);
    } else if j2 < j3 {
        return reorder3j(j1, j3, j2, m1, m3, m2, -sign);
    } else if m1 < 0 || m1 == 0 && m2 < 0 {
        return reorder3j(j1, j2, j3, -m1, -m2, -m3, -sign);
    } else {
        // sign doesn't matter if total J = j1 + j2 + j3 is even
        if (j1 + j2 + j3) % 2 == 0 {
            sign = 1;
        }
        return (j1, j2, j3, m1, m2, m3, sign);
    }
}

fn triangle_coefficient(j1: u32, j2: u32, j3: u32) -> Rational {
    let n1 = factorial(j1 + j2 - j3);
    let n2 = factorial(j1 - j2 + j3);
    let n3 = factorial(j2 + j3 - j1);
    let numerator = n1 * n2 * n3;
    let denominator = factorial(j1 + j2 + j3 + 1);

    let mut result = Rational::new(numerator, denominator);
    result.simplify();
    return result;
}

fn max(a: i32, b: i32, c: i32) -> i32 {
    std::cmp::max(a, std::cmp::max(b, c))
}

fn min(a: i32, b: i32, c: i32) -> i32 {
    std::cmp::min(a, std::cmp::min(b, c))
}

/// compute the sum appearing in the 3j symbol
fn compute_3j_series(beta1: i32, beta2: i32, beta3: i32, alpha1: i32, alpha2: i32) -> (f64, PrimeFactorization) {
    let range = max(alpha1, alpha2, 0)..(min(beta1, beta2, beta3) + 1);

    let mut numerators = Vec::with_capacity(range.len());
    let mut denominators = Vec::with_capacity(range.len());
    for k in range {
        let numerator = if k % 2 == 0 {
            PrimeFactorization::one()
        } else {
            PrimeFactorization::minus_one()
        };
        numerators.push(numerator);

        debug_assert!(k >= 0);
        let mut denominator = factorial(k as u32);

        debug_assert!((k - alpha1) >= 0);
        denominator *= factorial((k - alpha1) as u32);

        debug_assert!((k - alpha2) >= 0);
        denominator *= factorial((k - alpha2) as u32);

        debug_assert!((beta1 - k) >= 0);
        denominator *= factorial((beta1 - k) as u32);

        debug_assert!((beta2 - k) >= 0);
        denominator *= factorial((beta2 - k) as u32);

        debug_assert!((beta3 - k) >= 0);
        denominator *= factorial((beta3 - k) as u32);

        denominators.push(denominator);
    }

    let denominator = common_denominator(&mut numerators, &denominators);
    let mut numerator = 0.0;

    for num in numerators {
        numerator += num.as_f64();
    }

    return (numerator, denominator);
}

/// Given a list of numerators and denominators, compute the common denominator
/// and the rescaled numerator, putting all fractions at the same common
/// denominator
fn common_denominator(
    numerators: &mut [PrimeFactorization],
    denominators: &[PrimeFactorization]
) -> PrimeFactorization {
    debug_assert_eq!(numerators.len(), denominators.len());
    if denominators.is_empty() {
        return PrimeFactorization::one()
    }

    let mut denominator = denominators[0].clone();
    for other in denominators.iter().skip(1) {
        denominator.least_common_multiple(other);
    }

    // rescale numerators
    for (num, den) in numerators.iter_mut().zip(denominators.iter()) {
        *num *= &denominator;
        *num /= den;
    }

    return denominator;
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_ulps_eq;

    #[test]
    fn test_wigner3j() {
        // checked against sympy
        assert_ulps_eq!(wigner_3j(2, 6, 4, 0, 0, 1), 0.0);
        assert_ulps_eq!(wigner_3j(2, 6, 4, 0, 0, 0), f64::sqrt(715.0) / 143.0);
        assert_ulps_eq!(wigner_3j(5, 3, 2, -3, 3, 0), f64::sqrt(330.0) / 165.0);
        assert_ulps_eq!(wigner_3j(5, 3, 2, -2, 3, -1), -f64::sqrt(330.0) / 330.0);
        assert_ulps_eq!(wigner_3j(100, 100, 100, 100, -100, 0), 2.689688852311291e-13);

        assert_ulps_eq!(wigner_3j(0, 1, 1, 0, 0, 0), -0.5773502691896257);
    }

    #[test]
    fn test_clebsch_gordan() {
        // checked against sympy
        assert_ulps_eq!(clebsch_gordan(2, 0, 6, 0, 4, 1), 0.0);
        assert_ulps_eq!(clebsch_gordan(1, 1, 1, 1, 2, 2), 1.0);
        assert_ulps_eq!(clebsch_gordan(2, 2, 1, -1, 3, 1), f64::sqrt(1.0 / 15.0));
    }
}
