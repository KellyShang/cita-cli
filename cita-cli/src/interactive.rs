use std::env;
use std::io;
use std::iter;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use ansi_term::Colour::{Red, Yellow};
use clap::{self, ArgMatches};
use linefeed::complete::{Completer, Completion};
use linefeed::terminal::Terminal;
use linefeed::{Interface, Prompter, ReadResult};
use shell_words;

use cli::{abi_processor, build_interactive, key_processor, rpc_processor};
use printer::Printer;

const ASCII_WORD: &'static str = r#"
   ._____. ._____.  _. ._   ._____. ._____.   ._.   ._____. ._____.
   | .___| |___. | | | | |  |___. | |_____|   |_|   |___. | |_____|
   | |     ._. | | | |_| |  ._. | |   ._.   ._____. ._. | | ._____.
   | |     | | |_| \_____/  | | |_/   | |   | ,_, | | | |_/ |_____|
   | |___. | | ._.   ._.    | |       | |   | | | | | |     ._____.
   |_____| |_| |_|   |_|    |_|       |_|   |_| |_| |_|     |_____|
"#;

/// Interactive command line
pub fn start(url: &str) -> io::Result<()> {
    let interface = Arc::new(Interface::new("cita-cli")?);
    let mut url = url.to_string();
    let mut env_variable = GlobalConfig::new();

    let mut history_file = env::home_dir().unwrap();
    history_file.push(".cita-cli.history");
    let history_file = history_file.to_str().unwrap();

    let mut parser = build_interactive();

    interface.set_completer(Arc::new(CitaCompleter::new(parser.clone())));
    let style = Red.bold();
    let text = "cita> ";

    interface.set_prompt(&format!(
        "\x01{prefix}\x02{text}\x01{suffix}\x02",
        prefix = style.prefix(),
        text = text,
        suffix = style.suffix()
    ));

    if let Err(e) = interface.load_history(history_file) {
        if e.kind() == io::ErrorKind::NotFound {
            println!(
                "History file {} doesn't exist, not loading history.",
                history_file
            );
        } else {
            eprintln!("Could not load history file {}: {}", history_file, e);
        }
    }

    let mut printer = Printer::default();

    println!("{}", Red.bold().paint(ASCII_WORD));
    env_variable.print(&url);
    while let ReadResult::Input(line) = interface.read_line()? {
        let args = shell_words::split(line.as_str()).unwrap();

        if let Err(err) = match parser.get_matches_from_safe_borrow(args) {
            Ok(matches) => {
                match matches.subcommand() {
                    ("switch", Some(m)) => {
                        m.value_of("host").and_then(|host| {
                            url = host.to_string();
                            Some(())
                        });
                        if m.is_present("color") {
                            env_variable.switch_color();
                        }

                        if m.is_present("json") {
                            printer.switch_format();
                            env_variable.switch_format();
                        }

                        if m.is_present("debug") {
                            env_variable.switch_debug();
                        }

                        #[cfg(feature = "blake2b_hash")]
                        {
                            if m.is_present("algorithm") {
                                env_variable.switch_encryption();
                            }
                        }
                        #[cfg(not(feature = "blake2b_hash"))]
                        {
                            if m.is_present("algorithm") {
                                println!("[{}]", Red.paint("The current version does not support the blake2b algorithm. \
                                                    Open 'blak2b' feature and recompile cita-cli, please."));
                            }
                        }
                        env_variable.print(&url);
                        Ok(())
                    }
                    ("rpc", Some(m)) => {
                        rpc_processor(m, &printer, Some(url.as_str()), &env_variable)
                    }
                    ("abi", Some(m)) => abi_processor(m, &printer),
                    ("key", Some(m)) => key_processor(m, &printer, &env_variable),
                    ("info", _) => {
                        env_variable.print(&url);
                        Ok(())
                    }
                    ("exit", _) => break,
                    _ => Ok(()),
                }
            }
            Err(err) => Err(format!("{}", err)),
        } {
            printer.eprintln(&format!("{}", err), true);
        }

        interface.add_history_unique(line.clone());
        if let Err(err) = interface.save_history(history_file) {
            eprintln!("Save command history failed: {}", err);
        };
    }

    Ok(())
}

fn get_complete_strings<'a, 'b, 'p>(
    app: &'p clap::App<'a, 'b>,
    filter: Vec<&String>,
) -> Vec<String> {
    let mut strings: Vec<String> = vec![];
    strings.extend(
        app.p
            .subcommands()
            .map(|app| app.p.meta.name.clone())
            .collect::<Vec<String>>(),
    );
    strings.extend(
        app.p
            .flags()
            .map(|a| {
                let mut strings = vec![];
                a.s.short.map(|s| strings.push(format!("-{}", s)));
                a.s.long.map(|s| strings.push(format!("--{}", s)));
                strings
            })
            .fold(vec![], |mut all, part| {
                all.extend(part);
                all
            }),
    );
    strings.extend(
        app.p
            .opts()
            .map(|a| {
                let mut strings = vec![];
                a.s.short.map(|s| strings.push(format!("-{}", s)));
                a.s.long.map(|s| strings.push(format!("--{}", s)));
                strings
            })
            .fold(vec![], |mut all, part| {
                all.extend(part);
                all
            }),
    );
    strings
        .into_iter()
        .filter(|s| !filter.contains(&s))
        .collect()
}

fn _get_command_chain(matches: &ArgMatches) -> Vec<String> {
    let mut matches = Some(matches);
    let mut names: Vec<String> = vec![];
    while let Some(m) = matches {
        matches = m.subcommand_name()
            .map(|name| {
                names.push(name.to_owned());
                m.subcommand_matches(name)
            })
            .unwrap_or(None);
    }
    names
}

struct CitaCompleter<'a, 'b>
where
    'a: 'b,
{
    clap_app: clap::App<'a, 'b>,
}

impl<'a, 'b> CitaCompleter<'a, 'b> {
    fn new(clap_app: clap::App<'a, 'b>) -> Self {
        CitaCompleter { clap_app }
    }

    fn find_subcommand<'s, 'p, Iter: iter::Iterator<Item = &'s str>>(
        app: &'p clap::App<'a, 'b>,
        mut prefix_names: iter::Peekable<Iter>,
    ) -> Option<&'p clap::App<'a, 'b>> {
        if let Some(name) = prefix_names.next() {
            for inner_app in &(app.p.subcommands) {
                if inner_app.p.meta.name == name {
                    if prefix_names.peek().is_none() {
                        return Some(inner_app);
                    }
                    return Self::find_subcommand(inner_app, prefix_names);
                }
            }
        }
        None
    }
}

unsafe impl<'a, 'b> ::std::marker::Sync for CitaCompleter<'a, 'b> {}
unsafe impl<'a, 'b> ::std::marker::Send for CitaCompleter<'a, 'b> {}

impl<'a, 'b, Term: Terminal> Completer<Term> for CitaCompleter<'a, 'b> {
    fn complete(
        &self,
        word: &str,
        prompter: &Prompter<Term>,
        start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        let line = prompter.buffer();
        let mut args = shell_words::split(&line[..start]).unwrap();
        let root = args.clone();
        let filter = root.iter()
            .filter(|s| s.starts_with("-"))
            .collect::<Vec<&String>>();
        if let Some(cmd) = root.first() {
            match cmd.as_str() {
                "abi" => args.truncate(3),
                _ => args.truncate(2),
            }
        }
        let current_app = if args.is_empty() {
            Some(&self.clap_app)
        } else {
            Self::find_subcommand(&self.clap_app, args.iter().map(|s| s.as_str()).peekable())
        };
        if let Some(current_app) = current_app {
            let strings = get_complete_strings(current_app, filter);
            let mut target: Option<String> = None;
            if &strings
                .iter()
                .filter(|s| {
                    let matched = s.to_lowercase().contains(&word.to_lowercase());
                    if matched {
                        target = Some(s.to_string());
                    }
                    matched
                })
                .count() == &1
            {
                return Some(vec![Completion::simple(target.unwrap())]);
            }

            if !strings.is_empty() {
                return Some(
                    strings
                        .into_iter()
                        .filter(|s| {
                            if word.is_empty() {
                                true
                            } else {
                                s.starts_with(&word)
                            }
                        })
                        .map(|s| Completion::simple(s))
                        .collect::<Vec<Completion>>(),
                );
            }
        }
        None
    }
}

pub struct GlobalConfig {
    blake2b: bool,
    color: bool,
    debug: bool,
    json_format: bool,
    path: PathBuf,
}

impl GlobalConfig {
    pub fn new() -> Self {
        GlobalConfig {
            blake2b: false,
            color: true,
            debug: false,
            json_format: true,
            path: env::current_dir().unwrap(),
        }
    }

    #[cfg(feature = "blake2b_hash")]
    fn switch_encryption(&mut self) {
        self.blake2b = !self.blake2b;
    }

    fn switch_color(&mut self) {
        self.color = !self.color;
    }

    fn switch_debug(&mut self) {
        self.debug = !self.debug;
    }

    fn switch_format(&mut self) {
        self.json_format = !self.json_format;
    }

    pub fn blake2b(&self) -> bool {
        self.blake2b
    }

    pub fn color(&self) -> bool {
        self.color
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    fn print(&self, url: &str) {
        let path = self.path.to_string_lossy();
        let encryption = if self.blake2b {
            "blake2b_hash"
        } else {
            "sha3_hash"
        };
        let color = self.color.to_string();
        let debug = self.debug.to_string();
        let json = self.json_format.to_string();
        let values = [
            ("url", url),
            ("pwd", path.deref()),
            ("color", color.as_str()),
            ("debug", debug.as_str()),
            ("json", json.as_str()),
            ("encryption", encryption),
        ];

        let max_width = values
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(20) + 2;
        let output = values
            .iter()
            .map(|(name, value)| {
                format!(
                    "[{:^width$}]: {}",
                    name,
                    Yellow.paint(*value),
                    width = max_width
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        println!("{}", output);
    }
}
