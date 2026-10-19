mod display;
mod system;

fn main() {
    let info = system::collect();
    display::render(&info);
}
