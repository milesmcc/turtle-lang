use crate::parser::Rule;
use ansi_term::{Color, Style};
use pest::iterators::Pair;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Source {
    text: String,
    location: String,
}

impl Source {
    pub fn new(text: String, location: String) -> Self {
        Self { text, location }
    }
}

#[derive(Debug, Clone)]
pub struct SourcePosition {
    start_pos: usize,
    end_pos: usize,
    text: Arc<RwLock<Source>>,
}

impl SourcePosition {
    pub fn new(start_pos: usize, end_pos: usize, text: Arc<RwLock<Source>>) -> Self {
        Self {
            start_pos,
            end_pos,
            text,
        }
    }

    pub fn from_pair(pair: &Pair<'_, Rule>, source: &Arc<RwLock<Source>>) -> Self {
        Self::new(
            pair.as_span().start_pos().pos(),
            pair.as_span().end_pos().pos(),
            source.clone(),
        )
    }

    pub fn location(&self) -> Option<String> {
        match self.text.read() {
            Ok(text) => Some(text.location.clone()),
            Err(_) => None,
        }
    }
}

impl fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source = match self.text.read() {
            Ok(text) => text,
            Err(_) => return Err(fmt::Error),
        };
        let line_number =
            &source.text[0..self.start_pos]
                .chars()
                .fold(0, |acc, c| match c == '\n' {
                    true => acc + 1,
                    false => acc,
                })
                + 1;

        let lines = source.text.split('\n');

        let mut relevant_lines_formatted: Vec<(usize, String)> = Vec::new();

        let mut chars_seen = 0;
        for (i, line) in lines.enumerate() {
            let eol_pos = chars_seen + line.len() + 1;
            if self.start_pos < eol_pos && self.end_pos > chars_seen {
                let mut inner_start_pos: isize = self.start_pos as isize - chars_seen as isize;
                if inner_start_pos < 0 {
                    inner_start_pos = 0;
                }

                let mut inner_end_pos = self.end_pos - chars_seen;
                if inner_end_pos > line.len() {
                    inner_end_pos = line.len();
                }
                if inner_start_pos as usize != inner_end_pos && !line.is_empty() {
                    relevant_lines_formatted.push((
                        i + 1,
                        format!(
                            "{}{}{}",
                            &line[0..inner_start_pos as usize],
                            Color::Purple
                                .paint(&line[inner_start_pos as usize..inner_end_pos as usize]),
                            &line[inner_end_pos..]
                        ),
                    ));
                }
            }
            chars_seen += line.len() + 1;
        }
        fn indent(n: usize) -> String {
            String::from_utf8(vec![b' '; n]).unwrap()
        }

        let mut indentation = format!("{}", line_number).len() + 2;
        if indentation < 6 {
            indentation = 6;
        }

        writeln!(
            f,
            "{}{} {}",
            indent(indentation),
            Color::Blue.bold().paint("├"),
            Style::default()
                .dimmed()
                .paint(format!("{}:{} ↴", source.location, line_number)),
        )?;

        for (line_no, line) in relevant_lines_formatted {
            let line_no_str = format!("{}", line_no);
            let line_no_indentation = indent(indentation - line_no_str.len() - 1);
            writeln!(
                f,
                "{}{} {}",
                line_no_indentation,
                Color::Blue.bold().paint(format!("{} │", line_no_str)),
                line
            )?;
        }
        write!(f, "")
    }
}

impl Source {
    pub fn location(&self) -> String {
        self.location()
    }
}