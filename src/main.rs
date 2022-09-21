use minesweeper::{MineField};
use std::{io::{stdin,stdout,Write}};
use MSError::*;

enum MSError {
    InvalidCmdErr(String),
    ParseNumErr(String),
    MissingArg(String),
    FloatRangeErr(String, f32, f32),
    IntRangeErr(String, i32, i32)
}

fn main() {
    use MSError::*;

    let mut user_input = String::new();
    let mut field = MineField::new(15, 15);

    let mut quit = false;
    println!("{}", field);
    while !quit {
        
        print!(">");
        stdout().flush();
        user_input.clear();
        stdin().read_line(&mut user_input).expect("Did not enter a valid string");

        match proccess_command(&strip_trailing_newline(&user_input), &mut field) {
            Ok(exit) => { 
                if exit { break }
                else { println!("{}", field) }
            },
            Err(ParseNumErr(bad_str)) => eprintln!("Unable to parse to int \"{}\"", bad_str),
            Err(InvalidCmdErr(bad_cmd)) => eprintln!("Invalid command \"{}\"", bad_cmd),
            Err(MissingArg(miss_arg)) => eprintln!("Missing arg \"{}\"", miss_arg),
            Err(FloatRangeErr(arg, low, upp)) => eprintln!("Arg \"{}\" must be in the range {:.1} to {:.1}", arg, low, upp),
            Err(IntRangeErr(arg, low, upp)) => eprintln!("Arg \"{}\" must be in the range {} to {}", arg, low, upp),
        };
    } 
    
    //field.sweep(field.width()/2 - 1, field.height()/2);
    //field.neighbors(0, 0).iter().for_each(|tile| println!("{:?}", tile));
    //field.get_perimiter(&sweeped).iter().for_each(|tile| {field.flag(tile.x, tile.y);});
    //println!("{}", field);
    
}

fn proccess_command(cmd: &str, field: &mut  MineField) -> Result<bool, MSError> {

    let mut split = cmd.split_whitespace();
    if let Some(arg0) = split.next() {
        match arg0.to_lowercase().as_str() {
            "exit" => return Ok(true),
            "seed" => {
                if let Some(arg1) = split.next() {
                    let seed  = parse_int(arg1)? as u64;
                        field.set_seed(seed);
                        println!("set seed to {}", arg1);
                } else {
                    return Err(MissingArg("seed <int>".to_string()))
                }
            },
            "s" | "sweep" | "f" | "flag" => {
                if let Some(x_str) = split.next() {
                    let x = parse_int(x_str)?  - 1;
                    if let Some(y_str) = split.next() {
                        let y = field.height() as i32 - parse_int(y_str)?;
                        
                        if arg0.chars().next().unwrap() == 's' {
                            if let Some(tile) = field.sweep(x as usize, y as usize).iter().next() {
                                if tile.is_bomb() {
                                    field.reveal();
                                }
                            }
                        } else {
                            field.flag(x as usize, y as usize);
                        }
                        
                        
                    } else {
                        return Err(MissingArg(format!("{} {} <y>", arg0, x_str)))
                    }
                } else {
                    return Err(MissingArg(format!("{} <x> <y>", arg0)))
                }
            },
            "d" | "density" => {
                if let Some(d_str) = split.next() {
                    let d = parse_float(d_str)?;
                    if d < 0.0 || d > 1.0 {
                        return Err(FloatRangeErr("<d>".to_string(), 0.0, 1.0))
                    }
                } else {
                    return Err(MissingArg("density <d>".to_string()))
                }
            },
            "c" | "count" => {
                if let Some(d_str) = split.next() {
                    let c = parse_int(d_str)?;
                    if c < 0 || c > field.area() as i32 {
                        return Err(IntRangeErr("<c>".to_string(), 0, field.area() as i32))
                    }
                } else {
                    return Err(MissingArg("count <c>".to_string()))
                }
            },
            "r" | "reset" => field.reset(),
            _ => return Err(InvalidCmdErr(arg0.to_string())),
        };
    }   

    Ok(false)
}

fn strip_trailing_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(input)
}

fn parse_int(str: &str) -> Result<i32, MSError> {
    str.parse::<i32>().or(Err(ParseNumErr(str.to_string())))
}

fn parse_float(str: &str) -> Result<f32, MSError> {
    str.parse::<f32>().or(Err(ParseNumErr(str.to_string())))
}