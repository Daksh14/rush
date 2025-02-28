/*
A quick write-up on rush builtins:
Builtins are commands that are included with the shell. They are not able to be removed or modified without recompiling the shell.
Normally, a child process, such as a shell command, does not have direct access to the parent process's environment variables and other state.
However, the builtins are an exception to this rule. They are able to access the data because they are trusted to safely modify it.
Users are free to create their own builtins if they wish to modify the source code, but it comes with an inherent risk.

You may notice that builtin commands are referenced in commands::Runnable::Internal. An 'Internal' is essentially a function pointer to a builtin command.
An 'External' will only have access to its arguments and environment variables, but not the shell's state, mostly for security reasons.
 */

use std::env;
use std::fs;
use std::io::{BufRead, BufReader};

use colored::Colorize;

use crate::commands::{Context, StatusCode};
use crate::path;

pub fn test(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("{}", "Test command!".yellow());
        StatusCode::success()
    } else {
        eprintln!("Usage: test");
        StatusCode::new(1)
    }
}

pub fn exit(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        std::process::exit(0);
    } else {
        eprintln!("Usage: exit");
        StatusCode::new(1)
    }
}

pub fn working_directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("{}", context.cwd());
        StatusCode::success()
    } else {
        eprintln!("Usage: working-directory");
        StatusCode::new(1)
    }
}

pub fn change_directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 1 {
        match context.env_mut().set_path(args[0]) {
            Ok(_) => {
                // ! This might be better to have happen automatically
                context.env_mut().update_process_env_vars();
                StatusCode::success()
            }
            Err(_) => {
                eprintln!("Invalid path: '{}'", args[0]);
                StatusCode::new(2)
            }
        }
    } else {
        eprintln!("Usage: change-directory <path>");
        StatusCode::new(1)
    }
}

// TODO: Break up some of this code into different functions
pub fn list_directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    let files_and_directories = match args.len() {
        // Use the working directory as the default path argument
        // This uses expect() because it needs to crash if the working directory is invalid,
        // though in the future the error should be handled properly
        0 => fs::read_dir(env::current_dir().expect("Failed to get working directory"))
            .expect("Failed to read directory"),
        1 => {
            // Path::from_str_path() will attempt to expand and canonicalize the path, and return None if the path does not exist
            let absolute_path = match path::resolve(args[0], context.home()) {
                Some(path) => path,
                None => {
                    eprintln!("Invalid path: '{}'", args[0]);
                    return StatusCode::new(2);
                }
            };

            match fs::read_dir(&absolute_path) {
                Ok(files_and_directories) => files_and_directories,
                Err(_) => {
                    eprintln!(
                        "Failed to read directory: '{}'",
                        absolute_path.to_string_lossy().to_string()
                    );
                    return StatusCode::new(3);
                }
            }
        }
        _ => {
            eprintln!("Usage: list-directory <path>");
            return StatusCode::new(1);
        }
    };

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for fd in files_and_directories {
        let fd = fd.expect("Failed to read directory");

        let fd_name = fd
            .file_name()
            .to_str()
            .expect("Failed to read file name")
            .to_string();

        // TODO: Add a flag to show hidden files
        if fd_name.starts_with('.') {
            continue;
        }

        if fd.file_type().expect("Failed to read file type").is_dir() {
            // Append a '/' to directories
            let fd_name = format!("{}/", fd_name).bright_green().to_string();
            directories.push(fd_name)
        } else {
            files.push(fd_name)
        };
    }

    directories.sort();
    files.sort();

    for directory in directories {
        println!("{}", directory);
    }

    for file in files {
        println!("{}", file);
    }

    StatusCode::success()
}

// TODO: Find a better name for this
pub fn go_back(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        let prev_dir = match context.env().previous_working_directory.clone() {
            Some(dir) => dir,
            None => {
                eprintln!("No previous working directory available");
                return StatusCode::new(2);
            }
        }
        .to_string_lossy()
        .to_string();

        match context.env_mut().set_path(prev_dir.as_str()) {
            Ok(_) => {
                context.env_mut().update_process_env_vars();
                StatusCode::success()
            }
            Err(_) => {
                eprintln!("Invalid path: '{}'", prev_dir);
                StatusCode::new(3)
            }
        }
    } else {
        eprintln!("Usage: go-back");
        StatusCode::new(1)
    }
}

pub fn clear_terminal(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        // * "Magic" ANSI escape sequence to clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        StatusCode::success()
    } else {
        eprintln!("Usage: clear-terminal");
        StatusCode::new(1)
    }
}

pub fn create_file(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 1 {
        match fs::File::create(args[0]) {
            Ok(_) => StatusCode::success(),
            Err(_) => {
                eprintln!("Failed to create file: '{}'", args[0]);
                StatusCode::new(2)
            }
        }
    } else {
        eprintln!("Usage: create-file <path>");
        StatusCode::new(1)
    }
}

pub fn create_directory(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 1 {
        match fs::create_dir(args[0]) {
            Ok(_) => StatusCode::success(),
            Err(_) => {
                eprintln!("Failed to create directory: '{}'", args[0]);
                StatusCode::new(2)
            }
        }
    } else {
        eprintln!("Usage: create-directory <path>");
        StatusCode::new(1)
    }
}

pub fn delete_file(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 1 {
        match fs::remove_file(args[0]) {
            Ok(_) => StatusCode::success(),
            Err(_) => {
                eprintln!("Failed to delete file: '{}'", args[0]);
                StatusCode::new(2)
            }
        }
    } else {
        eprintln!("Usage: delete-file <path>");
        StatusCode::new(1)
    }
}

pub fn read_file(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    let file_name = match args.len() {
        1 => args[0].to_string(),
        _ => {
            eprintln!("Usage: read-file <path>");
            return StatusCode::new(1);
        }
    };

    let file = match fs::File::open(&file_name) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Failed to open file: '{}'", file_name);
            return StatusCode::new(2);
        }
    };

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        println!("{}", line);
    }

    StatusCode::success()
}

pub fn truncate(context: &mut Context, args: Vec<&str>) -> StatusCode {
    let truncation = match args.len() {
        0 => 1,
        // ! This is copilot code, it is extremely unsafe
        1 => match args[0].parse::<usize>() {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Invalid truncation length: '{}'", args[0]);
                return StatusCode::new(2);
            }
        },
        _ => {
            eprintln!("Usage: truncate <length (default 1)>");
            return StatusCode::new(1);
        }
    };

    context.cwd_mut().set_truncation(truncation);
    StatusCode::success()
}

pub fn untruncate(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        context.cwd_mut().disable_truncation();
        StatusCode::success()
    } else {
        eprintln!("Usage: untruncate");
        StatusCode::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::Shell;

    #[test]
    fn test_command_test_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = test(&mut context, Vec::new());

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_exit_success() {
        // * This is a placeholder test because the exit command
        // * will exit the program, effectively ending the test
    }

    #[test]
    fn test_command_working_directory_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = working_directory(&mut context, Vec::new());

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_change_directory_success_1() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = change_directory(&mut context, vec!["/"]);

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_change_directory_success_2() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = change_directory(&mut context, vec!["~"]);

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_change_directory_success_3() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        change_directory(&mut context, vec!["~"]);
        // ! This is not guaranteed to exist on the tester's system
        let status_code = change_directory(&mut context, vec!["Documents"]);

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_change_directory_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = change_directory(&mut context, vec!["/invalid/path"]);

        assert_eq!(status_code, StatusCode::new(2));
    }

    #[test]
    fn test_command_list_directory_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = list_directory(&mut context, Vec::new());

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_list_directory_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = list_directory(&mut context, vec!["/invalid/path"]);

        assert_eq!(status_code, StatusCode::new(2));
    }

    #[test]
    fn test_command_go_back_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        context.env_mut().set_path("/");
        let status_code = go_back(&mut context, Vec::new());

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_go_back_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = go_back(&mut context, Vec::new());

        assert_eq!(status_code, StatusCode::new(2));
    }

    #[test]
    fn test_command_truncate_success_1() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = truncate(&mut context, Vec::new());

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_truncate_success_2() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = truncate(&mut context, vec!["10"]);

        assert_eq!(status_code, StatusCode::success());
    }

    #[test]
    fn test_command_truncate_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status_code = truncate(&mut context, vec!["-10"]);

        assert_eq!(status_code, StatusCode::new(2));
    }
}
