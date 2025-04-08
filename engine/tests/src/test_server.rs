use server::api::{handle_calc, handle_about};
//use models::train::generate_beta_random_numbers;

#[test]
fn test_handle_calc() {
    //let data = generate_beta_random_numbers(10, 2.0, 2.0);
    // println!("data: {:?}", data);
    let res = handle_calc(true, vec![], 0.0, 1.0, 1000);
    println!("{:#?}", res);
}

#[test]
fn test_handle_about() {
    let res = handle_about();
    println!("{:?}", res);
}
