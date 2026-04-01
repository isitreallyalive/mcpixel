use mcpixel::process;

fn main() {
    let image = image::open("image.jpeg").unwrap();
    process(image, 64).unwrap().save("image.png").unwrap();
}
