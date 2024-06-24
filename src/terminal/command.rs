use nom::{bytes::complete::take_while1, character::complete::space0, IResult};

pub enum Command {
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
