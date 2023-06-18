


trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

struct Rectangle {
    width: f64,
    height: f64,
}

struct Circle {
    radius: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
}

fn print_shape_info(shape: &impl Shape) {
    println!("Area: {}", shape.area());
    println!("Perimeter: {}", shape.perimeter());
}

fn main() {
    let rectangle = Rectangle { width: 3.0, height: 4.0 };
    let circle = Circle { radius: 2.5 };

    print_shape_info(&rectangle);
    print_shape_info(&circle);
}