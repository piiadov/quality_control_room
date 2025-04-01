use approx::assert_abs_diff_eq;
use server::api::{handle_calc, handle_about, hypergeometric_pmf, quality, quant_quality};

#[test]
fn test_quality() {
    let mut q = quality(300, 5, 4, 10);
    assert_abs_diff_eq!(q[0], 0.33, epsilon = 1e-2);
    assert_abs_diff_eq!(q[1], 0.99, epsilon = 1e-2);

    q = quality(300, 15, 10, 10);
    assert_abs_diff_eq!(q[0], 0.40, epsilon = 1e-2);
    assert_abs_diff_eq!(q[1], 0.87, epsilon = 1e-2);

    q = quality(3_000, 150, 100, 10);
    assert_abs_diff_eq!(q[0], 0.58, epsilon = 1e-2);
    assert_abs_diff_eq!(q[1], 0.74, epsilon = 1e-2);

    q = quality(30_000, 1_500, 1_000, 10);
    assert_abs_diff_eq!(q[0], 0.64, epsilon = 1e-2);
    assert_abs_diff_eq!(q[1], 0.69, epsilon = 1e-2);
}

#[test]
fn test_hypergeometric_pmf() {
    assert_abs_diff_eq!(hypergeometric_pmf(300, 200, 15, 10), 0.2198, epsilon = 1e-4);
    assert_abs_diff_eq!(
        hypergeometric_pmf(30_000, 20_000, 1_500, 1_000),
        0.0224,
        epsilon = 1e-4
    );
}

#[test]
fn test_quant_quality() {
    let data = vec![
        200.0, 215.0, 210.0, 210.0, 240.0, 205.0, 210.0, 230.0, 250.0, 240.0,
    ];
    let n_total = data.len() as u64 * 50; // n_total >> n for small sampling
    let (x, q) = quant_quality(n_total, data);
    println!("{:?}", x);
    println!("{:?}", q);
}

#[test]
fn test_handle_calc() {
    let data = vec![
        //200.0, 215.0, 210.0, 210.0, 240.0, 205.0, 210.0, 230.0, 250.0, 240.0,
        1.0
    ];

    let res = handle_calc(data);
    println!("{:?}", res.q);
}

#[test]
fn test_handle_about() {
    let res = handle_about();
    println!("{:?}", res);
}
