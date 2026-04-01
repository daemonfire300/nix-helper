use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
    process,
};

#[derive(Default)]
struct State {
    in_fence: bool,
    in_table: bool,
    table_depth: i32,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("bionic-typst: {err}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let input = args.next().unwrap_or_else(|| usage(2));
    let output = args.next();
    if args.next().is_some() {
        usage(2);
    }

    let reader: Box<dyn BufRead> = if input == "-" {
        Box::new(BufReader::new(io::stdin().lock()))
    } else {
        Box::new(BufReader::new(File::open(Path::new(&input))?))
    };

    let writer: Box<dyn Write> = match output.as_deref() {
        None | Some("-") => Box::new(BufWriter::new(io::stdout().lock())),
        Some(path) => Box::new(BufWriter::new(File::create(Path::new(path))?)),
    };

    transform(reader, writer)
}

fn usage(code: i32) -> ! {
    eprintln!("usage: bionic-typst <input.typ|-> [output.typ|-]");
    process::exit(code);
}

fn transform<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    let mut state = State::default();
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        writer.write_all(transform_line(&line, &mut state).as_bytes())?;
    }
    writer.flush()
}

fn transform_line(line: &str, state: &mut State) -> String {
    let trimmed = line.trim_start();

    if trimmed.starts_with("```") {
        state.in_fence = !state.in_fence;
        return line.to_owned();
    }
    if state.in_fence {
        return line.to_owned();
    }

    if state.in_table {
        state.table_depth += paren_delta(line);
        if state.table_depth <= 0 {
            state.in_table = false;
            state.table_depth = 0;
        }
        return line.to_owned();
    }

    if trimmed.starts_with("#table(") || trimmed.starts_with("#table (") {
        state.in_table = true;
        state.table_depth = paren_delta(line);
        if state.table_depth <= 0 {
            state.in_table = false;
            state.table_depth = 0;
        }
        return line.to_owned();
    }

    if trimmed.starts_with('#') {
        return line.to_owned();
    }

    bionicize_line(line)
}

fn paren_delta(line: &str) -> i32 {
    let mut delta = 0;
    let mut in_code = false;
    for ch in line.chars() {
        if ch == '`' {
            in_code = !in_code;
        } else if !in_code {
            if ch == '(' {
                delta += 1;
            } else if ch == ')' {
                delta -= 1;
            }
        }
    }
    delta
}

fn bionicize_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len() + line.len() / 5);
    let mut i = 0;
    let mut in_code = false;

    while i < chars.len() {
        if chars[i] == '`' {
            in_code = !in_code;
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if in_code {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if starts_with(&chars, i, "#strong[") {
            let start = i;
            i += "#strong[".chars().count();
            let mut depth = 1;
            while i < chars.len() {
                if chars[i] == '[' {
                    depth += 1;
                } else if chars[i] == ']' {
                    depth -= 1;
                }
                i += 1;
                if depth == 0 {
                    break;
                }
            }
            while chars
                .get(i)
                .is_some_and(|c| c.is_ascii_alphabetic() || *c == '\'')
            {
                i += 1;
            }
            out.extend(chars[start..i].iter().copied());
            continue;
        }

        let end = take_word(&chars, i);
        if end > i && left_ok(&chars, i) && right_ok(&chars, end) {
            let word_len = chars[i..end]
                .iter()
                .filter(|c| c.is_ascii_alphabetic())
                .count();
            if word_len > 1 {
                let split = prefix_len(word_len);
                out.push_str("#strong[");
                let mut seen = 0;
                for ch in &chars[i..end] {
                    if ch.is_ascii_alphabetic() {
                        seen += 1;
                    }
                    out.push(*ch);
                    if seen == split {
                        out.push(']');
                    }
                }
                if seen < split {
                    out.push(']');
                }
            } else {
                out.extend(chars[i..end].iter().copied());
            }
            i = end;
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

fn starts_with(chars: &[char], start: usize, needle: &str) -> bool {
    let mut i = start;
    for expected in needle.chars() {
        if chars.get(i).copied() != Some(expected) {
            return false;
        }
        i += 1;
    }
    true
}

fn take_word(chars: &[char], start: usize) -> usize {
    if !chars.get(start).is_some_and(|c| c.is_ascii_alphabetic()) {
        return start;
    }

    let mut i = start;
    while chars.get(i).is_some_and(|c| c.is_ascii_alphabetic()) {
        i += 1;
    }

    while chars.get(i) == Some(&'\'') && chars.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic()) {
        i += 1;
        while chars.get(i).is_some_and(|c| c.is_ascii_alphabetic()) {
            i += 1;
        }
    }

    i
}

fn left_ok(chars: &[char], start: usize) -> bool {
    start == 0 || !is_left_blocker(chars[start - 1])
}

fn right_ok(chars: &[char], end: usize) -> bool {
    end >= chars.len() || !is_right_blocker(chars[end])
}

fn is_left_blocker(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '@' | '#' | '/' | '<' | '*' | ':' | '-' | '.')
}

fn is_right_blocker(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '>' | '/' | '*' | ':' | '-' | '.')
}

fn prefix_len(n: usize) -> usize {
    match n {
        0 | 1 => n,
        2 => 1,
        3 | 4 => 2,
        5 | 6 => 3,
        7 => 6,
        _ => (n * 45).div_ceil(100),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_plain_text() {
        assert_eq!(
            bionicize_line("Infrastructure requirements\n"),
            "#strong[Infrast]ructure #strong[requir]ements\n"
        );
    }

    #[test]
    fn keeps_inline_code() {
        assert_eq!(
            bionicize_line("Use `nix copy` here\n"),
            "#strong[Us]e `nix copy` #strong[he]re\n"
        );
    }

    #[test]
    fn skips_existing_strong_markup() {
        assert_eq!(
            bionicize_line("#strong[Requir]ements stay\n"),
            "#strong[Requir]ements #strong[st]ay\n"
        );
    }
}
