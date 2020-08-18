use ansi_term::{Color, Style};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::Validator;
use rustyline::validate::{self, MatchingBracketValidator};
use rustyline::Editor;
use rustyline::{CompletionType, Config, Context};
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::sync::{Arc, RwLock};

use crate::{parse, CallSnapshot, Environment};

#[derive(Helper)]
struct ReplHelper {
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
    completer: FilenameCompleter,
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ReplHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ReplHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(Style::new().dimmed().paint(hint).to_string())
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for ReplHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

pub fn spawn(env: Arc<RwLock<Environment>>) {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .build();
    let h = ReplHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(h));

    if rl.load_history(".turtle_history.txt").is_err() {
        println!("It looks like this is your first time running Turtle from this directory; no history was loaded.")
    }

    loop {
        let p = format!("🐢 > ");
        rl.helper_mut().expect("No helper").colored_prompt =
            Color::Green.bold().paint(&p).to_string();
        let line = rl.readline(&p);
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match parse(line.as_str(), "<stdin>") {
                    Ok(values) => {
                        for value in values {
                            let snapshot = CallSnapshot::root(&value.clone());
                            match value.eval_async(snapshot, env.clone()).unwrap().recv().unwrap() {
                                Ok(result) => println!(
                                    "   {} {}",
                                    Color::Blue.bold().paint("="),
                                    Style::default().bold().paint(format!("{}", result))
                                ),
                                Err(error) => eprintln!("{}", error),
                            }
                        }
                    }
                    Err(err) => eprintln!("{:#}", err),
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
        rl.save_history(".turtle_history.txt").unwrap();
    }
}
