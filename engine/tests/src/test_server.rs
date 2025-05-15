use server::api::{handle_calc, handle_about};
use models::train::generate_random_data;
use models::train::DistributionType;

#[test]
fn test_handle_calc() {
    let data = generate_random_data(DistributionType::Beta, 10, [2.0, 2.0]);
    // println!("data: {:?}", data);
    let res = handle_calc(0, true, data, 0.0, 1.0,
                          3000, 10);
    println!("{:#?}", res);
}

#[test]
fn test_handle_about() {
    let res = handle_about();
    println!("{:?}", res);
}
