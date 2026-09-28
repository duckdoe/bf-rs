use std::{
    env,
    fs::read_to_string,
    io::{stdin, stdout, Read, Write},
};

struct File {
    contents: String,
}

#[derive(Eq, PartialEq)]
enum FileError {
    NameErr,
}

#[derive(PartialEq, Eq, Debug)]
enum Token {
    PutChar, //.
    GetChar, //,
    Lbrack,  // [
    Rbrack,  // ]
    Rarr,    // >
    Larr,    // <
    Sub,     // -
    Add,     // +
    Eof,     // \0
}

enum TokenizerError {
    UnableToTokenize,
    UnknownTokenError(char),
}

enum InterpreterError {
    MemoryBoundsError, // Used when
    UnclosedBrackError(usize, bool),
}

fn open_file(file_name: &str) -> Result<File, FileError> {
    let split_file_name = file_name.split_once(".").unwrap();
    if split_file_name.1 != "bf" {
        return Err(FileError::NameErr);
    }

    let contents = String::from(read_to_string(file_name).expect("File does not exist"));

    Ok(File { contents })
}

fn tokenize(file: File) -> Result<Vec<Token>, TokenizerError> {
    let mut tokens = vec![];
    let mut contents = file.contents.chars();

    loop {
        // for each loop-back we are going to generate a token
        let char = contents.next();

        match char {
            Some(char) => match char {
                '.' => {
                    tokens.push(Token::PutChar);
                }
                ',' => {
                    tokens.push(Token::GetChar);
                }
                '>' => {
                    tokens.push(Token::Rarr);
                }
                '<' => {
                    tokens.push(Token::Larr);
                }
                '[' => {
                    tokens.push(Token::Lbrack);
                }
                ']' => {
                    tokens.push(Token::Rbrack);
                }
                '-' => {
                    tokens.push(Token::Sub);
                }
                '+' => {
                    tokens.push(Token::Add);
                }
                '\0' => {
                    tokens.push(Token::Eof);
                }
                ch => {
                    if ch.is_ascii_whitespace() {
                        continue;
                    }
                    return Err(TokenizerError::UnknownTokenError(ch));
                }
            },
            None => {
                break;
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

fn interprete(tokens: Vec<Token>, memory: &mut [u8]) -> Result<(), InterpreterError> {
    let mut brack_depth = 0; // keeps track of how deep we are in brackets
    let mut pos = 0; // keeps track of token positioon
    let mut cur_token = &tokens[pos];
    let forward = true; // This tells us whether to scan forward or backward
                        // through the tokens
    let len = tokens.len();

    let mut memory_ptr = 0; // This is the memory pointer we give this to the user
                            // They do whatever they want with the memory size
                            // we give them
    let mem_size = memory.len();

    while pos < len - 1 {
        match cur_token {
            Token::Rarr => {
                let ptr = memory_ptr + 1;

                if ptr > mem_size {
                    return Err(InterpreterError::MemoryBoundsError);
                }

                memory_ptr = ptr;
            }
            Token::Larr => {
                let ptr = memory_ptr - 1;

                #[allow(unused_comparisons)]
                if ptr < 0 {
                    return Err(InterpreterError::MemoryBoundsError);
                }

                memory_ptr = ptr;
            }
            Token::Lbrack => {
                brack_depth += 1; // This tells us we've moved one step lower in depth
                if memory[memory_ptr] == 0 {
                    while brack_depth > 0 && pos < len - 1 {
                        pos += 1;
                        cur_token = &tokens[pos];

                        if *cur_token == Token::Lbrack {
                            brack_depth += 1
                        } else if *cur_token == Token::Rbrack {
                            brack_depth -= 1;
                        }
                    }
                }
            }
            Token::Rbrack => {
                while memory[memory_ptr] != 0 && pos > 0 && brack_depth > 0 {
                    pos -= 1; // Move back through the code
                    cur_token = &tokens[pos];

                    if *cur_token == Token::Lbrack {
                        brack_depth -= 1;
                    } else if *cur_token == Token::Rbrack {
                        brack_depth += 1;
                    }
                }

                if memory[memory_ptr] != 0 && brack_depth == 0 {
                    pos -= 1;
                } else {
                    brack_depth -= 1; // This tells us that we dont have to move back
                                      // And that the '[' has beem terminated
                }
            }
            Token::Add => {
                memory[memory_ptr] = memory[memory_ptr].wrapping_add(1);
            }
            Token::Sub => {
                memory[memory_ptr] = memory[memory_ptr].wrapping_sub(1);
            }
            Token::PutChar => {
                let block_value = memory[memory_ptr];
                let char = char::from(block_value);

                print!("{char}");
            }
            Token::GetChar => {
                let mut byte = [0u8; 1];
                stdin().read_exact(&mut byte).unwrap();

                memory[memory_ptr] = byte[0];
            }
            Token::Eof => {
                break;
            }
        }

        pos = if forward && pos < len - 1 {
            pos + 1
        } else {
            pos - 1
        };
        cur_token = &tokens[pos];
    }

    if brack_depth != 0 {
        return Err(InterpreterError::UnclosedBrackError(brack_depth, forward));
    }

    Ok(())
}

fn main() {
    let cmd_args: Vec<String> = env::args().collect();

    if cmd_args.len() < 2 {
        eprintln!("Try cargo run [filename]");
        return;
    }

    let file = open_file(cmd_args[1].as_str());
    let mut tokens = Err(TokenizerError::UnableToTokenize);
    match file {
        Ok(file) => {
            tokens = tokenize(file);
        }
        Err(err) => {
            if err == FileError::NameErr {
                eprintln!("file name must contain the .bf extension");
                return;
            }
        }
    }

    let mut memory: [u8; 30_000] = [0; 30_000];

    match tokens {
        Ok(tokens) => {
            let result = interprete(tokens, &mut memory);

            if let Err(err) = result {
                match err {
                    InterpreterError::MemoryBoundsError => {
                        eprintln!("Oops seems like you moved away from memory scope")
                    }
                    InterpreterError::UnclosedBrackError(depth, direction) => {
                        if direction {
                            eprintln!("Unterminated '[' The error happened {depth} level(s) deep");
                        } else {
                            eprintln!("Unnecessary ']' The error happened {depth} level(s) deep");
                        }
                    }
                }
            } else {
                stdout().flush().expect("Something went horribly wrong");
            }
        }
        Err(err) => match err {
            TokenizerError::UnknownTokenError(ch) => {
                eprintln!("Unknown command '{ch}'");
            }
            TokenizerError::UnableToTokenize => {
                eprintln!("Unable to tokenize for some odd reason");
            }
        },
    }
}
