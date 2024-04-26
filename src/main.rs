use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
mod render;
mod gen;
mod tringulation;

fn gameloop() {
    loop {
        // maybe handle the events
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Unable to create event loop!");
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let state = pollster::block_on(render::State::new(&window));
}
