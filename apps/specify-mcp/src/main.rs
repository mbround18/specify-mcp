fn main() {
    let arg_count = std::env::args().count();
    let result = if arg_count <= 1 {
        specify_mcp::mcp::server::run_stdio_server()
    } else {
        specify_mcp::mcp::workflows::run_cli()
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
