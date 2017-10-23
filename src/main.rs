extern crate clap;
extern crate rand;
use clap::{App, Arg};
use std::fs;
use rand::{thread_rng, Rng};
use std::process::Command;
extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;



fn main() {

    let matches = App::new("turack")
        .version("1.0")
        .about("Grading assignments is fun!")
        .author("")
        .arg(
            Arg::with_name("dir")
                .help("Directory with assignments")
                .required(true)
                .index(1),
        )
        .get_matches();

    let dir = matches.value_of("dir").unwrap();

    let paths = fs::read_dir(dir).unwrap();

    fn get_name(e: &fs::DirEntry) -> String {
        let p = e.path();
        p.file_stem()
            .map(|x| x.to_str().unwrap())
            .unwrap_or("")
            .to_string()
    }

    let mut dirs: Vec<String> = vec![];
    for entry in paths
        .filter_map(|e| e.ok())
        .filter(|x| x.path().is_dir())
        .filter(|x| get_name(x) != "compiled")
    {
        dirs.push(entry.path().to_str().map(|x| x.to_string()).unwrap());
    }

    let mut rng = thread_rng();
    let mut dirs = dirs.as_mut_slice();
    rng.shuffle(dirs);

    let mut undone: Vec<String> = dirs.to_vec();
    let mut done: Vec<String> = vec![];

    fn grade(s: String) -> String {
        Command::new("drracket")
            .arg(format!("{}/handin.rkt", s))
            .spawn()
            .expect("Could not start DrRacket!");
        Command::new("drracket")
            .arg(format!("{}/grade-prefilled.rktd", s))
            .spawn()
            .expect("Could not start DrRacket!");
        s
    }

    // REPL
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(ref line) if line == "exit" || line == "quit" => {
                println!("Bye!");
                break;
            }
            Ok(ref line) if line == "next" => {
                let graded = grade(undone.pop().unwrap());
                done.push(graded);
                rl.add_history_entry(&line);
            }
            Ok(ref line) if line == "last" => {
                let last = if let Some(s) = done.pop() {
                    s
                } else {
                    println!("There is no last graded assignment.");
                    continue;
                };
                let graded = grade(last);
                done.push(graded);
                rl.add_history_entry(&line);
            }
            Ok(line) => {
                rl.add_history_entry(&line);
                println!("Only next, last and quit are available as commands.");
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
