fn main() {
    env_logger::init();
    std::process::exit(main_inner())
}

fn main_inner() -> i32 {
    oraide_language_server::run_server()
}