fn main() {
    env_logger::init();
    std::process::exit(main_inner())
}

fn main_inner() -> i32 {
    orals::run_server()
}