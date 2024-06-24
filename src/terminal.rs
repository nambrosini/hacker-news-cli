mod command;

use std::io::{self, stdout, Write};

use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode},
    style::{self, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand,
};

use crate::client::{fetch_ask, fetch_best, fetch_jobs, fetch_new, fetch_show, Item};

use crate::terminal::command::Command;

const MAX_TITLE_WIDTH: usize = 120;

#[derive(Default)]
pub struct Terminal {
    should_quit: bool,
    is_command: bool,
    last_line: u16,
    is_dashboard_active: bool,
}

impl Terminal {
    pub async fn run(&mut self) {
        self.initialize().unwrap();
        let result = self.repl().await;
        Self::terminate().unwrap();
        stdout().flush().unwrap();
        result.unwrap();
    }

    fn initialize(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        Self::clear_screen()?;
        self.show_dashboard()?;
        stdout.flush()
    }

    fn terminate() -> Result<(), std::io::Error> {
        disable_raw_mode()?;
        stdout().queue(MoveTo(0, 0))?;
        stdout().flush()
    }

    fn clear_screen() -> Result<(), std::io::Error> {
        let mut stdout = io::stdout();
        stdout.queue(Clear(ClearType::All))?;
        stdout.flush()
    }

    async fn repl(&mut self) -> Result<(), io::Error> {
        let mut input = String::new();

        loop {
            if event::poll(std::time::Duration::from_secs(1))? {
                if let Event::Key(event) = event::read()? {
                    match event.code {
                        KeyCode::Char(c) => {
                            if self.is_command {
                                input.push(c);
                            }
                            if c == ':' {
                                let (_, height) = terminal::size()?;
                                stdout().queue(MoveTo(0, height - 1))?;
                                self.is_command = true;
                            }
                            stdout().queue(style::PrintStyledContent(c.to_string().white()))?;
                            io::stdout().flush()?;
                            // // Print the character and append it to the input string
                            // stdout().queue(style::PrintStyledContent(c.to_string().white()))?;
                            // io::stdout().flush()?;
                            // input.push(c);
                        }
                        KeyCode::Enter => {
                            // User pressed Enter, break the loop
                            self.evaluate(&input).await?;
                            self.is_command = false;
                            input.clear();
                        }
                        KeyCode::Backspace => {
                            // Handle backspace
                            if !input.is_empty() {
                                // Remove last character from the input string
                                input.pop();
                                // Move the cursor back, print a space to erase the character, then move back again
                                io::stdout()
                                    .queue(crossterm::cursor::MoveLeft(1))?
                                    .queue(crossterm::style::Print(" "))?
                                    .queue(crossterm::cursor::MoveLeft(1))?;
                                io::stdout().flush()?;
                            } else {
                                self.is_command = false;
                            }
                        }
                        _ => {}
                    }
                }
                self.refresh_screen()?;
                if self.should_quit {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn evaluate(&mut self, buffer: &str) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        // stdout.queue(style::Print("\r\n"))?;

        let command = match Command::try_from(buffer) {
            Ok(command) => command,
            Err(e) => {
                stdout
                    .queue(Clear(ClearType::CurrentLine))?
                    .queue(MoveTo(0, self.last_line))?
                    .queue(style::PrintStyledContent("> ".blue()))?
                    .queue(style::Print(buffer))?
                    .queue(style::Print("\r\n"))?
                    .queue(style::PrintStyledContent(format!("Error: {e}. ").red()))?
                    .queue(style::Print("\r\n"))?
                    .queue(style::PrintStyledContent(
                        "Type ':help' (or <:?>) for usage.".yellow(),
                    ))?
                    .queue(style::Print("\r\n"))?;
                return io::stdout().flush();
            }
        };

        stdout
            .queue(MoveTo(0, self.last_line))?
            .queue(Clear(ClearType::All))?
            .queue(MoveTo(0, 0))?
            .queue(style::PrintStyledContent("> ".blue()))?
            .queue(style::Print(buffer))?
            .queue(style::Print("\r\n"))?;

        match command {
            Command::Top(count) => {
                let stories = fetch_best(count).await.unwrap();
                self.show_stories(&stories).await.unwrap();
            }
            Command::New(count) => {
                let stories = fetch_new(count).await.unwrap();
                self.show_stories(&stories).await.unwrap();
            }
            Command::Show(count) => {
                let stories = fetch_show(count).await.unwrap();
                self.show_stories(&stories).await.unwrap();
            }
            Command::Ask(count) => {
                let stories = fetch_ask(count).await.unwrap();
                self.show_stories(&stories).await.unwrap();
            }
            Command::Jobs(count) => {
                let stories = fetch_jobs(count).await.unwrap();
                self.show_stories(&stories).await.unwrap();
            }
            Command::Help => {
                stdout.queue(style::Print("Available commands:"))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::Print(
                    "  top <count> - Print the top <count> stories",
                ))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::Print(
                    "  new <count> - Print the newest <count> stories",
                ))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::Print(
                    "  show <id> - Show the details of the item with the given <id>",
                ))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::Print(
                    "  ask <count> - Print the top <count> ask HN stories",
                ))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::Print(
                    "  jobs <count> - Print the top <count> job stories",
                ))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::Print("  help - Show this help"))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::Print("  exit - Quit the application"))?;
            }
            Command::Exit => {
                self.should_quit = true;
            }
        }

        io::stdout().flush()
    }

    fn refresh_screen(&self) -> Result<(), io::Error> {
        if self.should_quit {
            Self::clear_screen()?;
            io::stdout().queue(style::Print("Goodbye.\r\n"))?;
        }
        Ok(())
    }

    async fn show_stories(&mut self, items: &[Item]) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        let (width, _) = terminal::size().unwrap();
        let (_, height) = cursor::position().unwrap();
        let mut new_height = height;
        let new_cursor_pos = width as usize / 2 - (MAX_TITLE_WIDTH / 2);
        stdout.queue(MoveTo(new_cursor_pos as u16, new_height))?;
        for item in items {
            stdout.queue(style::PrintStyledContent(
                item.title.clone().unwrap().yellow(),
            ))?;
            new_height += 1;
            stdout.queue(MoveTo(new_cursor_pos as u16, new_height))?;
            if let Some(url) = &item.url {
                stdout.queue(style::PrintStyledContent(url.clone().blue().underlined()))?;
                new_height += 2;
                stdout.queue(MoveTo(new_cursor_pos as u16, new_height))?;
            } else if let Some(text) = &item.text {
                stdout.queue(style::PrintStyledContent(text.clone().green()))?;
                new_height += 2;
                stdout.queue(MoveTo(new_cursor_pos as u16, new_height))?;
            }
        }
        stdout.queue(MoveTo(new_cursor_pos as u16, height))?;

        self.last_line = height;
        Ok(())
    }

    fn show_dashboard(&mut self) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        let (width, height) = terminal::size()?;
        let width = width / 2 - 27 / 2;
        let height = height / 4;
        stdout.queue(MoveTo(width, height))?;
        stdout.queue(style::Print("Welcome to Hacker News CLI!"))?;
        stdout.queue(MoveTo(width, height + 1))?;
        stdout.queue(style::Print("Type 'help' (or <?>) for usage.\r\n"))?;
        stdout.queue(MoveTo(width, height))?;
        self.is_dashboard_active = true;
        stdout.flush()
    }
}
