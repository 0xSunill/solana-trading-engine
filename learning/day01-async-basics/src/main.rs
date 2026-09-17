async fn get_price() -> f64 {
    100.0
}

fn main() {
    let price: f64 = get_price();

    println!("{}", price);
}