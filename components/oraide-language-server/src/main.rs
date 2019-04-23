// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

fn main() {
    env_logger::init();
    std::process::exit(main_inner())
}

fn main_inner() -> i32 {
    oraide_language_server::run_server()
}