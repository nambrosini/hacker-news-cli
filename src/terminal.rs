use std::{
    fmt::write,
    io::{self, stdout, Write},
};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    style::{self, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand,
};
use nom::{bytes::complete::take_while1, character::complete::space0, IResult};

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
                                io::stdout()
                                    .queue(crossterm::cursor::MoveLeft(1))?
                                    .queue(crossterm::style::Print(" "))?
                                    .queue(crossterm::cursor::MoveLeft(1))?;
                                io::stdout().flush()?;
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

        let command = match Command::try_from(buffer) {
            Ok(command) => command,
            Err(e) => {
                stdout.queue(style::PrintStyledContent(format!("Error: {e}. ").red()))?;
                stdout.queue(style::Print("\r\n"))?;
                stdout.queue(style::PrintStyledContent(
                    "Type 'help' (or <?>) for usage.".yellow(),
                ))?;
                stdout.queue(style::Print("\r\n"))?;
                Self::print_prompt()?;
                return io::stdout().flush();
            }
        };

        match command {
            Command::Top(count) => {
                stdout.queue(style::Print(format!("Printing top {count}")))?;
            }
            Command::New(count) => {
                stdout.queue(style::Print(format!("Printing new {count}")))?;
            }
            Command::Show(id) => {
                stdout.queue(style::Print(format!("Showing item {id}")))?;
            }
            Command::Ask(count) => {
                stdout.queue(style::Print(format!("Printing ask {count}")))?;
            }
            Command::Jobs(count) => {
                stdout.queue(style::Print(format!("Printing jobs {count}")))?;
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

enum Command {
    Top(usize),
    New(usize),
    Show(usize),
    Ask(usize),
    Jobs(usize),
    Help,
    Exit,
}

impl TryFrom<&str> for Command {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (remaining, command) =
            first_word(value).map_err(|_| format!("Invalid command: {value}"))?;
        match command {
            "top" => {
                let (_, param) =
                    first_word(remaining).map_err(|_| format!("Invalid command: {value}"))?;
                match param.parse::<usize>() {
                    Ok(count) => Ok(Self::Top(count)),
                    Err(e) => Err(format!("Invalid count: {e}")),
                }
            }
            "new" => {
                let (_, param) =
                    first_word(remaining).map_err(|_| format!("Invalid command: {value}"))?;
                match param.parse::<usize>() {
                    Ok(count) => Ok(Self::New(count)),
                    Err(e) => Err(format!("Invalid count: {e}")),
                }
            }
            "show" => {
                let (_, param) =
                    first_word(remaining).map_err(|_| format!("Invalid command: {value}"))?;
                match param.parse::<usize>() {
                    Ok(count) => Ok(Self::Show(count)),
                    Err(e) => Err(format!("Invalid count: {e}")),
                }
            }
            "ask" => {
                let (_, param) =
                    first_word(remaining).map_err(|_| format!("Invalid command: {value}"))?;
                match param.parse::<usize>() {
                    Ok(count) => Ok(Self::Ask(count)),
                    Err(e) => Err(format!("Invalid count: {e}")),
                }
            }
            "jobs" => {
                let (_, param) =
                    first_word(remaining).map_err(|_| format!("Invalid command: {value}"))?;
                match param.parse::<usize>() {
                    Ok(count) => Ok(Self::Jobs(count)),
                    Err(e) => Err(format!("Invalid count: {e}")),
                }
            }
            "help" | "?" => Ok(Self::Help),
            "exit" | "quit" => Ok(Self::Exit),
            _ => Err(format!("Invalid command: {value}")),
        }
    }
}

fn is_not_whitespace(c: char) -> bool {
    !c.is_whitespace()
}

fn first_word(input: &str) -> IResult<&str, &str> {
    let (input, word) = take_while1(is_not_whitespace)(input)?;
    let (input, _) = space0(input)?; // consume any trailing spaces (optional)
    Ok((input, word))
}
