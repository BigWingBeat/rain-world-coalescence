#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[no_mangle]
pub extern "C" fn create_point(x: i32, y: i32) -> Point {
    Point { x, y }
}

#[no_mangle]
pub extern "C" fn add_points(a: Point, b: Point) -> Point {
    Point {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}
