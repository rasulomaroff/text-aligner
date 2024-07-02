use std::{error::Error, fs::File, io::Write, slice::Iter};

pub fn run(
    content: &str,
    writer: &mut impl Writer,
    max_len: usize,
    align: &Align,
) -> Result<(), Box<dyn Error>> {
    let words = content.split(' ').collect::<Vec<&str>>();

    match align {
        Align::Left => {
            process(
                &|line: &Line, writer: &mut dyn Writer, _: usize| {
                    let mut words_iter = line.iter();
                    writer.write(words_iter.next().unwrap());

                    words_iter.for_each(|w| writer.write(&format!(" {w}")));
                },
                &words,
                writer,
                max_len,
            );
        }
        Align::Right => {
            process(
                &|line: &Line, writer: &mut dyn Writer, len: usize| {
                    let free_space = len - line.len;

                    for _ in 0..free_space {
                        writer.write(" ");
                    }

                    let mut words_iter = line.iter();
                    writer.write(words_iter.next().unwrap());

                    words_iter.for_each(|w| writer.write(&format!(" {w}")));
                },
                &words,
                writer,
                max_len,
            );
        }
        Align::Justify => process(
            &|line: &Line, writer: &mut dyn Writer, len: usize| {
                let free_space = len - line.len;

                let gaps = if line.words_count == 1 {
                    1
                } else {
                    line.words_count - 1
                };

                let big_jump = free_space / gaps;
                let mut small_jump = free_space % gaps;

                let mut words_iter = line.iter();
                writer.write(words_iter.next().unwrap());

                words_iter.for_each(|w| {
                    for _ in 0..big_jump {
                        writer.write(" ");
                    }

                    if small_jump != 0 {
                        writer.write(" ");
                        small_jump -= 1;
                    }

                    writer.write(&format!(" {w}"))
                });
            },
            &words,
            writer,
            max_len,
        ),
    }

    Ok(())
}

fn process(
    on_line_wrap: &impl Fn(&Line, &mut dyn Writer, usize) -> (),
    words: &Vec<&str>,
    writer: &mut impl Writer,
    max_len: usize,
) {
    let mut line = Line::new(max_len);
    let mut words_iter = words.iter();

    'line: loop {
        for word in &mut words_iter {
            match line.push(word) {
                LineState::NextWord => (),
                LineState::Wrap => {
                    on_line_wrap(&line, writer, max_len);

                    line.clear();
                    line.push(word);
                    writer.write("\n");

                    continue 'line;
                }
            };
        }

        if line.words_count > 0 {
            // remove last character as it's \n
            let last_word = line.words.last_mut().unwrap();
            let mut chars = last_word.chars();
            chars.next_back();
            line.len -= 1;
            *last_word = chars.as_str();

            // process the last line
            on_line_wrap(&line, writer, max_len);
            writer.write("\n");
        }

        break;
    }
}

enum LineState {
    Wrap,
    NextWord,
}

struct Line<'a> {
    words_count: usize,
    len: usize,
    max_len: usize,
    words: Vec<&'a str>,
}

impl<'a> Line<'a> {
    fn new(max_len: usize) -> Self {
        Self {
            words_count: 0,
            len: 0,
            max_len,
            words: Vec::new(),
        }
    }

    fn push<'b>(&mut self, word: &'b str) -> LineState
    where
        'b: 'a,
    {
        let word_len = word.len();

        if word_len > self.max_len {
            panic!("Got word's length that's longer than the the line length: {word}");
        }

        let word_len = if self.words_count != 0 {
            word_len + 1
        } else {
            word_len
        };

        if word_len + self.len <= self.max_len {
            self.words.push(word);
            self.len += word_len;
            self.words_count += 1;

            LineState::NextWord
        } else {
            LineState::Wrap
        }
    }

    fn clear(&mut self) {
        self.words_count = 0;
        self.len = 0;
        self.words.clear();
    }

    fn iter(&self) -> Iter<'a, &str> {
        self.words.iter()
    }
}

#[derive(Debug)]
pub enum Align {
    Left,
    Right,
    Justify,
}

pub trait Writer {
    fn write(&mut self, content: &str);
}

#[derive(Debug)]
pub struct StdoutWriter;

impl Writer for StdoutWriter {
    fn write(&mut self, content: &str) {
        print!("{content}");
    }
}

#[derive(Debug)]
pub struct FileWriter {
    pub file: File,
}

impl Writer for FileWriter {
    fn write(&mut self, content: &str) {
        self.file
            .write(content.as_bytes())
            .expect("Wasn't able to write to the file");
    }
}

pub struct Config<'a> {
    pub align: Align,
    pub len: usize,
    pub file_path: &'a str,
    pub destination_path: Option<&'a str>,
}

impl<'a> Config<'a> {
    pub fn build(args: &'a [String]) -> Result<Self, String> {
        let mut args = args.iter();

        // skipping the first argument as it's the program name
        args.next();

        let file_path = match args.next() {
            Some(f) => f,
            None => return Err("Didn't get a file path".to_string()),
        };

        let len = match args.next() {
            Some(l) => l.parse().expect("expected usize"),
            None => return Err("Didn't get a string length".to_string()),
        };

        let align = match args.next().map(|a| a.to_lowercase()) {
            Some(a) if a == LEFT => Align::Left,
            Some(a) if a == RIGHT => Align::Right,
            Some(a) if a == JUSTIFY => Align::Justify,
            Some(v) => {
                return Err(format!(
                    "Align option is incorrect. Expected `left`, `right`, `justify`, got: {v}"
                ))
            }
            None => return Err("Didn't get an align option".to_string()),
        };

        Ok(Self {
            len,
            align,
            file_path,
            destination_path: args.next().map(|s| s as &str),
        })
    }
}

const LEFT: &'static str = "left";
const RIGHT: &'static str = "right";
const JUSTIFY: &'static str = "justify";

#[cfg(test)]
mod tests {
    use crate::*;

    struct StringWriter {
        val: String,
    }

    impl Writer for StringWriter {
        fn write(&mut self, content: &str) {
            self.val.push_str(content);
        }
    }

    #[test]
    fn justifies_content() {
        let content = String::from("Hi there! My name is Roben Li.\n");
        let mut writer = StringWriter { val: String::new() };

        crate::run(&content, &mut writer, 10, &Align::Justify).unwrap();

        assert_eq!("Hi  there!\nMy name is\nRoben  Li.\n", writer.val);
    }

    #[test]
    fn aligns_left() {
        let content = String::from("Hello there! This text should be left-aligned.\n");
        let mut writer = StringWriter { val: String::new() };

        crate::run(&content, &mut writer, 15, &Align::Left).unwrap();

        assert_eq!(
            "Hello there!\nThis text\nshould be\nleft-aligned.\n",
            writer.val
        );
    }

    #[test]
    fn aligns_right() {
        let content = String::from("Gracias! And this text must be right-aligned.\n");
        let mut writer = StringWriter { val: String::new() };

        crate::run(&content, &mut writer, 15, &Align::Right).unwrap();

        assert_eq!(
            "   Gracias! And\n this text must\n             be\n right-aligned.\n",
            writer.val
        );
    }

    #[test]
    #[should_panic]
    fn fails_to_align_when_word_is_long() {
        let content = String::from("Gracias! And this text must be right-aligned.\n");
        let mut writer = StringWriter {
            val: String::from(""),
        };

        // "right-aligned." is 14 symbols where the line width is 10, `run` function should panic
        crate::run(&content, &mut writer, 10, &Align::Right).unwrap();
    }
}
