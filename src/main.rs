#![warn(clippy::all, clippy::pedantic)]
use std::io::{self, stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    style::{self, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand,
};

fn main() {
    Terminal::default().run();
}

#[derive(Default)]
pub struct Terminal {
    should_quit: bool,
}

impl Terminal {
    pub fn run(&mut self) {
        Self::initialize().unwrap();
        let result = self.repl();
        Self::terminate().unwrap();
        stdout().flush().unwrap();
        result.unwrap();
    }

    fn initialize() -> Result<(), std::io::Error> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        Self::clear_screen()?;
        stdout.queue(MoveTo(0, 0))?;
        Self::print_prompt()?;
        stdout.flush()
    }

    fn terminate() -> Result<(), std::io::Error> {
        disable_raw_mode()?;
        stdout().queue(MoveTo(0, 0))?;
        Ok(())
    }

    fn clear_screen() -> Result<(), std::io::Error> {
        let mut stdout = io::stdout();
        stdout.queue(Clear(ClearType::All))?;
        Ok(())
    }

    fn repl(&mut self) -> Result<(), io::Error> {
        let mut input = String::new();

        loop {
            if event::poll(std::time::Duration::from_secs(1))? {
                if let Event::Key(event) = event::read()? {
                    match event.code {
                        KeyCode::Char(c) => {
                            // Print the character and append it to the input string
                            stdout().queue(style::PrintStyledContent(c.to_string().white()))?;
                            io::stdout().flush()?;
                            input.push(c);
                        }
                        KeyCode::Enter => {
                            // User pressed Enter, break the loop
                            self.evaluate(&input)?;
                            input.clear();
                        }
                        KeyCode::Backspace => {
                            // Handle backspace
                            if !input.is_empty() {
                                // Remove last character from the input string
                                input.pop();
                                // Move the cursor back, print a space to erase the character, then move back again
                                execute!(
                                    io::stdout(),
                                    crossterm::cursor::MoveLeft(1),
                                    crossterm::style::Print(" "),
                                    crossterm::cursor::MoveLeft(1)
                                )?;
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

    fn evaluate(&mut self, buffer: &str) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        stdout.queue(style::Print("\r\n"))?;
        if buffer.trim().starts_with("top") {
            let split: Vec<&str> = buffer.split_whitespace().collect();
            if split.len() == 2 {
                let count = split[1].parse::<usize>().unwrap_or(10);
                stdout.queue(style::Print(format!("Printing top {count}")))?;
            }
        } else if buffer.trim() == "exit" || buffer.trim() == "quit" {
            self.should_quit = true;
        } else {
            stdout.queue(style::Print(buffer))?;
        }
        stdout.queue(style::Print("\r\n"))?;
        Self::print_prompt()?;

        io::stdout().flush()
    }

    fn print_prompt() -> Result<(), io::Error> {
        io::stdout().queue(style::PrintStyledContent("> ".blue()))?;
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), io::Error> {
        if self.should_quit {
            Self::clear_screen()?;
            io::stdout().queue(style::Print("Goodbye.\r\n"))?;
        }
        Ok(())
    }
}
