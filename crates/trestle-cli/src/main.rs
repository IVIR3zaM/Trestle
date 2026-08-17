fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--version" => {
                let version = env!("CARGO_PKG_VERSION");
                let target = env!("CARGO_TRESTLE_TARGET_TRIPLE");
                let sha = env!("CARGO_TRESTLE_GIT_SHA");
                println!("trestle {} {} {}", version, target, sha);
            }
            "--help" | "-h" => {
                print_usage();
            }
            _ => {
                print_usage();
                std::process::exit(1);
            }
        }
    } else {
        print_usage();
    }
}

fn print_usage() {
    eprintln!("Usage: trestle [OPTIONS]");
    eprintln!("       trestle --version");
    eprintln!("       trestle --help");
}
