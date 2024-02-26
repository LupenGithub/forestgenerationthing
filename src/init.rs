mod window_init;

pub fn initialize(){
    env_logger::init();
    pollster::block_on(window_init::init_window());
}
