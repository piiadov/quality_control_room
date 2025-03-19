use lib::wrapper::*;

#[test]
fn test_shuffle() {
    let vec: Vec<i32> = vec![0,0,0,0,0];
    c_shuffle(&vec);
    println!("Shuffle vector: {:?}", vec);
}

#[test]
fn test_split_data() {
    let x: Vec<[f64; 4]> = (0..10*4).step_by(4)
        .map(|i| [i as f64, (i + 1) as f64, (i + 2) as f64, (i + 3) as f64]).collect();
    let y: Vec<[f64; 2]> = (0..10*2).step_by(2)
        .map(|i| [i as f64, (i + 1) as f64]).collect();

    println!("x: {:?}", x);
    println!("y: {:?}", y);

    let x = flat_vector::<4>(x);
    let y = flat_vector::<2>(y);

    println!("flat x: {:?}", x);
    println!("flat y: {:?}", y);

    let (x_train, y_train,
        x_test, y_test) = c_split_data(x, y, 10,
                                       4, 2, 0.75);
    println!("flat x_test: {:?}", x_test);
    println!("flat y_test: {:?}", y_test);

    let x_test = shape_vector::<4>(x_test);
    let y_test = shape_vector::<2>(y_test);

    println!("x_test: {:?}", x_test);
    println!("y_test: {:?}", y_test);
}
