fn main() {
    println!("Hello, diffr!");
    greet("User");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: &str) {
    println!("Welcome, {}!", name);
}
