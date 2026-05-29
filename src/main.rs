mod algo;
mod app;
mod backend;
mod log;
mod model;
mod safety;
mod tui;
mod util;

fn main() -> std::io::Result<()> {
    app::runtime::run()
}
