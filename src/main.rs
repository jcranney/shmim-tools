use risio::Accessor;

fn main() {
    println!("Hello, world!");
    let mut image = risio::RawImage::<f64>::create_new("bob", &[10,10]).unwrap();
    image.array_mut()[0] = 15.0;
}
