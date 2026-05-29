use crossterm::event::{self, Event};
use std::io;

use crate::{
    app::{
        cli::{parse_args, Launch},
        state::App,
    },
    tui,
};

pub(crate) fn run() -> io::Result<()> {
    let launch = parse_args(std::env::args()).map_err(|err| {
        eprintln!("{err}");
        eprintln!("Try 'rfdisk --help' for more information.");
        io::Error::new(io::ErrorKind::InvalidInput, err)
    })?;

    let Launch::Run(lang) = launch else {
        return Ok(());
    };

    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, lang);
    ratatui::restore();
    result
}

fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    lang: crate::util::i18n::Lang,
) -> io::Result<()> {
    let mut app = App::new_with_lang(lang);

    loop {
        terminal.draw(|frame| tui::render::ui(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if crate::app::event::handle_key_event(&mut app, key.code, key.kind) {
                break;
            }
        }
    }

    Ok(())
}
