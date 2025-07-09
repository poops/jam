use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    queue,
    style::{Attribute, Print, PrintStyledContent, SetAttribute, Stylize},
    terminal,
};
use mpd::status::State;
use mpd::Client;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() {
    let mut conn = Client::connect("127.0.0.1:6600").expect("error connecting to mpd");
    let (mut _w, mut h) = terminal::size().expect("error getting terminal size");

    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode().expect("error enabling raw mode");

    loop {
        if event::poll(Duration::from_millis(500)).expect("error polling event") {
            match event::read().expect("error reading event") {
                Event::Resize(_nw, nh) => {
                    // w = nw;
                    h = nh;
                }
                Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        let _ = match event.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('>') => conn.next(),
                            KeyCode::Char('<') => conn.prev(),
                            KeyCode::Char('s') => conn.stop(),
                            KeyCode::Char('p') => conn.toggle_pause(),
                            _ => Ok(()),
                        }
                        .map_err(|e| eprintln!("{e}"));
                    }
                }
                _ => {}
            }
        }

        let status = match conn.status() {
            Ok(status) => match status.state {
                State::Pause => PrintStyledContent("Paused: ".yellow()),
                State::Play => PrintStyledContent("Playing: ".green()),
                State::Stop => PrintStyledContent("Stopped: ".red()),
            },
            Err(e) => {
                eprintln!("error getting status from mpd: {e}");
                PrintStyledContent("".red())
            }
        };

        let _ = queue!(
            stdout,
            cursor::MoveTo(0, h),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )
        .map_err(|e| eprintln!("error clearing screen: {e}"));

        if let Some(song) = conn.currentsong().expect("error getting song from mpd") {
            let _ = queue!(
                stdout,
                status,
                SetAttribute(Attribute::Bold),
                Print(song.title.unwrap_or("Unknown".to_string())),
                SetAttribute(Attribute::Reset),
                Print(" by ".dark_grey()),
                Print(song.artist.unwrap_or("Unknown".to_string())),
            )
            .map_err(|e| eprintln!("error printing song: {e}"));
        }

        let _ = stdout
            .flush()
            .map_err(|e| eprintln!("error flushing stdout: {e}"));

        thread::sleep(Duration::from_millis(16));
    }

    terminal::disable_raw_mode().expect("error disabling raw mode");
}
