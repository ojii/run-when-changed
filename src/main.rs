use clap::Clap;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::process::{exit, Command};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Clap)]
#[clap(version = "1.0")]
struct Arguments {
    #[clap(short, long)]
    recursive: bool,
    #[clap(short, long)]
    immediate: bool,
    #[clap(short, long)]
    stop_on_error: bool,
    #[clap(short, long, default_value = "0")]
    rate_limit: u64,
    path: String,
    #[clap(multiple = true)]
    command: Vec<String>,
}

fn main() {
    let args: Arguments = Arguments::parse();

    let (tx, rx) = channel();
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(args.rate_limit)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    let recursive_mode = if args.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };
    watcher.watch(&args.path, recursive_mode).unwrap();

    if args.immediate {
        run(&args)
    }

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("{:?}", event);
                match event {
                    DebouncedEvent::NoticeWrite(_)
                    | DebouncedEvent::NoticeRemove(_)
                    | DebouncedEvent::Create(_)
                    | DebouncedEvent::Write(_)
                    | DebouncedEvent::Remove(_)
                    | DebouncedEvent::Rename(_, _) => run(&args),
                    _ => (),
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn run(args: &Arguments) {
    let command = args.command.first().unwrap();
    let arguments = args.command.iter().skip(1);
    match Command::new(command).args(arguments).status() {
        Ok(_) => println!("run successful"),
        Err(_) => {
            println!("failed");
            if args.stop_on_error {
                exit(1);
            }
        }
    }
}
