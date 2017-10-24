extern crate clap;
extern crate rand;
use clap::{App, Arg};
use std::fs;
use rand::{thread_rng, Rng};
use std::process::Command;
extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::io::Error;

pub trait StripMargin {
    fn strip_margin(self) -> String;
}

impl StripMargin for &'static str {
    fn strip_margin(self) -> String {
        let mut out = Vec::new();
        for l in self.lines().filter(|x| !x.is_empty()) {
            for s in l.splitn(2, '|').nth(1) {
                out.push(s);
            }
        }
        out.join("\n")
    }
}

static SAVE_FILE: &'static str = "turack_progress.txt";

fn main() {

    let help_text: String = "|Commands available from the prompt:
       |
       |  next           Open next assignment.
       |  last           Open last assignment.
       |  commit         Copy grade-prefilled.rktd to grade.rktd for all done assignments.
       |  commit all     Same as above but include undone assignments.
       |  feierabend     Is it feierabend yet?
       |  restart        Change status of all assignments from done to undone.
       |  quit           .
       |  help           Display this list of commands."
        .strip_margin();
    let matches = App::new("turack")
        .version("1.0")
        .about("Grading assignments is fun!")
        .author("")
        .arg(
            Arg::with_name("dir-name")
                .help("Directory with assignments")
                .required(true)
                .index(1),
        )
        .get_matches();

    let dir = matches.value_of("dir-name").unwrap();

    let paths = fs::read_dir(dir).unwrap();

    fn get_name(e: &fs::DirEntry) -> String {
        let p = e.path();
        p.file_stem()
            .map(|x| x.to_str().unwrap())
            .unwrap_or("")
            .to_string()
    }

    fn commit(s: &String) {
        match fs::copy(
            format!("{}/grade-prefilled.rktd", s),
            format!("{}/grade.rktd", s),
        ) {
            Ok(_) => println!("Committed {}", s),
            _ => println!("Could not commit {}", s),
        }
    }

    fn save(done: &Vec<String>, undone: &Vec<String>, dir: &str) {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::create(format!("{}/{}", dir, SAVE_FILE)).unwrap();
        for s in done.iter() {
            file.write_all(s.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
        file.write_all(b"DONE\n").unwrap();
        for s in undone.iter() {
            file.write_all(s.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
    }

    fn load(dir: &str) -> Result<(Vec<String>, Vec<String>), Error> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut done: Vec<String> = vec![];
        let mut undone: Vec<String> = vec![];

        let mut file = File::open(format!("{}/{}", dir, SAVE_FILE))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let mut is_undone = true;
        for l in contents.lines() {
            if is_undone {
                if l == "DONE" {
                    is_undone = false;
                    continue;
                }
                println!("Done: {}", l);
                done.push(l.to_string());
            } else {
                println!("Undone: {}", l);
                undone.push(l.to_string());
            }
        }


        Ok((done, undone))

    }

    let mut dirs: Vec<String> = vec![];
    for entry in paths
        .filter_map(|e| e.ok())
        .filter(|x| x.path().is_dir())
        .filter(|x| get_name(x) != "compiled")
    {
        dirs.push(entry.path().to_str().map(|x| x.to_string()).unwrap());
    }

    let mut undone: Vec<String> = shuffle(dirs);
    let mut done: Vec<String> = vec![];

    fn shuffle(mut v: Vec<String>) -> Vec<String> {
        let mut rng = thread_rng();
        let mut v = v.as_mut_slice();
        rng.shuffle(v);
        v.to_vec()
    }

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
                rl.add_history_entry(&line);
                let graded = grade(undone.pop().unwrap());
                done.push(graded);
            }
            Ok(ref line) if line == "last" => {
                rl.add_history_entry(&line);
                let last = if let Some(s) = done.pop() {
                    s
                } else {
                    println!("There is no last graded assignment.");
                    continue;
                };
                let graded = grade(last);
                done.push(graded);
            }
            Ok(ref line) if line == "commit" => {
                rl.add_history_entry(&line);
                for dir in done.iter() {
                    commit(&dir);
                }
            }
            Ok(ref line) if line == "commit all" => {
                rl.add_history_entry(&line);
                for dir in done.iter() {
                    commit(&dir);
                }
                for dir in undone.iter() {
                    commit(&dir);
                }
            }
            Ok(ref line) if line == "restart" => {
                rl.add_history_entry(&line);
                undone.append(&mut done);
                undone = shuffle(undone);
            }
            Ok(ref line) if line == "feierabend" => {
                rl.add_history_entry(&line);
                let num_undone = undone.len();
                if num_undone == 0 {
                    println!("You've done it! Have a good one!")
                } else {
                    println!(
                        "It's not quite feierabend yet. Just {} assignments left.",
                        num_undone
                    );
                }
            }
            Ok(ref line) if line == "save" => {
                rl.add_history_entry(&line);
                save(&done, &undone, &dir);
            }
            Ok(ref line) if line == "load" => {
                rl.add_history_entry(&line);
                match load(&dir) {
                    Ok((tmp_done, tmp_undone)) => {
                        if !(tmp_done.is_empty())  {
                            done = tmp_done;
                            undone = tmp_undone;
                            undone = shuffle(undone);
                        }
                    }
                    Err(e) => println!("{}/{} {}", dir, SAVE_FILE, e),
                }
            }
            Ok(ref line) if line == "help" => {
                rl.add_history_entry(&line);
                println!("{}", help_text);
            }
            Ok(line) => {
                rl.add_history_entry(&line);
                println!("Use 'help' to get more information about the available commands.");
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
