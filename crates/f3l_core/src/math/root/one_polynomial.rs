
use num_traits::Float;


/// x^2 + `factor_0` * x + `factor_1` = 0
pub fn root2<T: Float>(factor: [T; 2]) -> [T; 2] {
    let inner = (factor[0].powi(2) - T::from(4f32).unwrap() * factor[1]).max(T::zero());
    let mut out = [
        T::from(0.5f32).unwrap() * (-factor[0] - inner.sqrt()),
        T::from(0.5f32).unwrap() * (-factor[0] + inner.sqrt()),
    ];
    if out[0] > out[1] {
        out.swap(0, 1);
    }
    out
}

/// Ref: [wiki](https://en.wikipedia.org/wiki/Cubic_equation)<br>
/// x^3 + `factor_0` * x^2 + `factor_1` * x^ + `factor_2` = 0
pub fn root3<T: Float>(factor: [T; 3]) -> [T; 3] {
    let [b, c, d] = factor;
    if d == T::zero() {
        let qua = root2([b, c]);
        let mut out = [T::zero(), qua[0], qua[1]];
        out.sort_by(|&a, b| a.partial_cmp(b).unwrap());
        return out;
    }

    let beta = c / T::from(3f32).unwrap() - b.powi(2) / T::from(9f32).unwrap();
    let alpha = -b.powi(3) / T::from(27f32).unwrap() - d / T::from(2f32).unwrap() + b * c / T::from(6f32).unwrap();

    let front = -b / T::from(3f32).unwrap();
    let mul_factor = T::from(2).unwrap() * (-beta).sqrt();

    let theta_factor = alpha / (-beta).powf(T::from(1.5).unwrap());
    let theta_factor = theta_factor.max(-T::one());
    let theta_factor = theta_factor.min(T::one());

    let x1 = front + mul_factor * (theta_factor.acos() / T::from(3f32).unwrap()).cos();
    let x2 = front + mul_factor * ((theta_factor.acos() + T::from(2f32 * std::f32::consts::PI).unwrap()) / T::from(3f32).unwrap()).cos();
    let x3 = front + mul_factor * ((theta_factor.acos() - T::from(2f32 * std::f32::consts::PI).unwrap()) / T::from(3f32).unwrap()).cos();

    let mut out = [x1, x2, x3];
    out.sort_by(|&a, b| a.partial_cmp(b).unwrap());
    out
}

/// Ref: [wiki_zh](https://zh.wikipedia.org/zh-tw/%E4%B8%89%E6%AC%A1%E6%96%B9%E7%A8%8B)
pub fn root3_eq2<T: Float>(factor: [T; 3]) -> [T; 3] {
    let [b, c, d] = factor;
    if d == T::zero() {
        let qua = root2([b, c]);
        let mut out = [T::zero(), qua[0], qua[1]];
        out.sort_by(|&a, b| a.partial_cmp(b).unwrap());
        return out;
    }

    let a_ = b.sqrt() - T::from(3f32).unwrap() * c;
    let b_ = b * c - T::from(9).unwrap() * d;
    let c_ = c.powi(2) - T::from(3f32).unwrap() * b * d;
    let tri = b_.powi(2) - T::from(4).unwrap() * a_ * c_;

    let mut out = if a_ == b_ {
        let ans = -c / b;
        return [ans, ans, ans];
    } else if tri > T::zero() {
        let top = tri.sqrt();
        let (y1, y2) = (
            (a_ * b + T::from(3f32).unwrap() * (-b_ - top) / T::from(2f32).unwrap()).powf(T::from(1. / 3f32).unwrap()),
            (a_ * b + T::from(3f32).unwrap() * (-b_ + top) / T::from(2f32).unwrap()).powf(T::from(1. / 3f32).unwrap()),
        );
        [
            (-b - (y1 + y2)) / T::from(3f32).unwrap(),
            (-T::from(2f32).unwrap() * b + (y1 + y2) - T::from(3f32.sqrt()).unwrap() * (y1 - y2)) / T::from(6f32).unwrap(),
            (-T::from(2f32).unwrap() * b + (y1 + y2) + T::from(3f32.sqrt()).unwrap() * (y1 - y2)) / T::from(6f32).unwrap(),
        ]
    } else if tri == T::zero() {
        let k = b_ / a_;
        [
            -b + k, -k * T::from(0.5f32).unwrap(), -k * T::from(0.5f32).unwrap()
        ]
    } else {
        let t = (T::from(2f32).unwrap() * a_ * b - T::from(3f32).unwrap() * b_) / (T::from(2f32).unwrap() * a_ * a_.sqrt());
        let theta = t.min(T::one()).max(-T::one()).acos() / T::from(3f32).unwrap();
        let cs = theta.cos();
        let ss = theta.sin();
        let a_sqrt = a_.sqrt();
        [
            (-b - T::from(2f32).unwrap() * a_sqrt * cs) / T::from(3f32).unwrap(),
            (-b + a_sqrt * (cs - T::from(3f32.sqrt()).unwrap() * ss)) / T::from(3f32).unwrap(),
            (-b + a_sqrt * (cs + T::from(3f32.sqrt()).unwrap() * ss)) / T::from(3f32).unwrap(),
        ]
    };
    out.sort_by(|&a, b| a.partial_cmp(b).unwrap());
    out
}

#[test]
fn test_root2() {
    // x^2 - 5 * x + 4 = 0
    let factor = [-5f32, 4f32];
    let answer = [1f32, 4f32];

    let target = root2(factor);
    assert_eq!(answer, crate::round_slice_n(target, 4));
}

#[test]
fn test_root3() {
    // x^3 + 0 * x^2 -15 * x -4 = 0
    let factor = [0f32, -15., -4.];
    let answer = crate::round_slice_n([-3.732050807568877, -0.26794919243112064, 4f32], 4);

    let target1 = crate::round_slice_n(root3(factor), 4);
    let target2 = crate::round_slice_n(root3_eq2(factor), 4);

    assert_eq!(answer, target1);
    assert_eq!(answer, target2);
}

#[test]
fn test_root3_d0() {
    // x^3 + 0 * x^2 -15 * x -4 = 0
    let factor = [0f32, -15., 0.];
    let answer = [-3.873, 0f32, 3.873];

    let target = root3(factor);

    assert_eq!(crate::round_slice_n(answer, 4), crate::round_slice_n(target, 4));
}

#[test]
fn test_root3_by_eigenvalues() {
    use crate::matrix3x3::compute_eigenvalues;
    use f3l_glam::glam::{Mat3, Vec3};

    let factor = [0f32, -15., -4.];

    // a x^3 + b x^2 + c x + d = 0
    // | 0  0  -d/a | 
    // | 1  0  -c/a | 
    // | 0  1  -b/a |

    let mat = Mat3::from_cols(
        Vec3::new(0., 1., 0.),
        Vec3::new(0., 0., 1.),
        Vec3::new(-factor[2], -factor[1], -factor[0])
    );

    let mut answer = crate::round_slice_n([-3.732050807568877, -0.26794919243112064, 4f32], 4);
    let mut target = crate::round_slice_n(compute_eigenvalues::<f32>(mat), 4);
    
    answer.sort_by(|&a, b| a.partial_cmp(b).unwrap());
    target.sort_by(|&a, b| a.partial_cmp(b).unwrap());

    assert_eq!(answer, target);
}