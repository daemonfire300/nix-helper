use std::collections::HashSet;
use std::env;
use std::fmt;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::process::{Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::thread;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage: nixos-rebuild-helper [nixos-rebuild args...]");
        return ExitCode::from(2);
    }

    match run(&args) {
        Ok(summary) => {
            let code: u8 = summary.exit_code_as_u8();
            ExitCode::from(code)
        }
        Err(error) => {
            eprintln!("nixos-rebuild-helper: {error}");
            ExitCode::from(1)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Ok,
    NoChanges,
    Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StreamKind {
    Stdout,
    Stderr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineKind {
    Normal,
    Highlight,
    Warning,
    Error,
}

#[derive(Debug, Eq, PartialEq)]
struct RunSummary {
    status: Status,
    exit_code: i32,
    saw_change: bool,
    saw_failure_hint: bool,
    highlights: Vec<String>,
}

impl RunSummary {
    fn exit_code_as_u8(&self) -> u8 {
        self.exit_code.clamp(0, u8::MAX as i32) as u8
    }
}

const MAX_HIGHLIGHTS: usize = 8;
const CHANGE_PREFIXES: &[&str] = &[
    "would restart the following units:",
    "would start the following units:",
    "stopping the following units:",
    "reloading the following units:",
    "restarting the following units:",
    "starting the following units:",
    "the following new units were started:",
    "activating the configuration...",
    "restarting systemd...",
    "restarting sysinit-reactivation.target",
];

const FAILURE_SUBSTRINGS: &[&str] = &["error:", "failed"];

struct Analyzer {
    saw_change: bool,
    saw_failure_hint: bool,
    highlights: Vec<String>,
    seen_highlights: HashSet<String>,
}

impl Analyzer {
    fn new() -> Self {
        Self {
            saw_change: false,
            saw_failure_hint: false,
            highlights: Vec::new(),
            seen_highlights: HashSet::new(),
        }
    }

    fn observe_line(&mut self, _stream: StreamKind, line: &str) -> LineKind {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if CHANGE_PREFIXES
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
        {
            self.saw_change = true;
            self.push_highlight(trimmed);
            return LineKind::Highlight;
        }

        if FAILURE_SUBSTRINGS
            .iter()
            .any(|needle| lower.contains(needle))
        {
            self.saw_failure_hint = true;
            self.push_highlight(trimmed);

            if lower.contains("warning:") {
                return LineKind::Warning;
            }

            return LineKind::Error;
        }

        if lower.contains("warning:") {
            return LineKind::Warning;
        }

        LineKind::Normal
    }

    fn finish(self, exit_code: i32) -> RunSummary {
        let status = if exit_code != 0 || self.saw_failure_hint {
            Status::Failure
        } else if self.saw_change {
            Status::Ok
        } else {
            Status::NoChanges
        };

        RunSummary {
            status,
            exit_code,
            saw_change: self.saw_change,
            saw_failure_hint: self.saw_failure_hint,
            highlights: self.highlights,
        }
    }

    fn push_highlight(&mut self, line: &str) {
        if line.is_empty() || self.highlights.len() >= MAX_HIGHLIGHTS {
            return;
        }

        let owned = line.to_owned();
        if self.seen_highlights.insert(owned.clone()) {
            self.highlights.push(owned);
        }
    }
}

#[derive(Clone, Copy)]
struct Renderer {
    color_stdout: bool,
    color_stderr: bool,
}

impl Renderer {
    fn new() -> Self {
        Self {
            color_stdout: io::stdout().is_terminal(),
            color_stderr: io::stderr().is_terminal(),
        }
    }

    #[cfg(test)]
    fn plain() -> Self {
        Self {
            color_stdout: false,
            color_stderr: false,
        }
    }

    fn print_live_line(&self, stream: StreamKind, kind: LineKind, line: &str) -> io::Result<()> {
        match stream {
            StreamKind::Stdout => {
                let mut out = io::stdout().lock();
                writeln!(out, "{}", self.decorate(line, kind, self.color_stdout))
            }
            StreamKind::Stderr => {
                let mut err = io::stderr().lock();
                writeln!(err, "{}", self.decorate(line, kind, self.color_stderr))
            }
        }
    }

    fn render_summary(&self, summary: &RunSummary) -> String {
        let mut output = String::new();
        let status_text = match summary.status {
            Status::Ok => "OK",
            Status::NoChanges => "NO CHANGES",
            Status::Failure => "FAILURE",
        };

        output.push_str("== nixos-rebuild-helper ==\n");
        output.push_str(&format!("Status: {status_text}\n"));
        output.push_str(&format!("Exit code: {}\n", summary.exit_code));

        if summary.highlights.is_empty() {
            if summary.status == Status::NoChanges {
                output.push_str("Highlights:\n");
                output.push_str("- no change markers were observed\n");
            }
        } else {
            output.push_str("Highlights:\n");
            for line in &summary.highlights {
                output.push_str("- ");
                output.push_str(line);
                output.push('\n');
            }
        }

        output
    }

    fn print_summary(&self, summary: &RunSummary) -> io::Result<()> {
        let mut err = io::stderr().lock();
        write!(err, "{}", self.render_summary(summary))
    }

    fn decorate(&self, line: &str, kind: LineKind, color_enabled: bool) -> String {
        if !color_enabled {
            return line.to_owned();
        }

        let code = match kind {
            LineKind::Normal => return line.to_owned(),
            LineKind::Highlight => "1;34",
            LineKind::Warning => "1;33",
            LineKind::Error => "1;31",
        };

        format!("\x1b[{code}m{line}\x1b[0m")
    }
}

fn run(args: &[String]) -> Result<RunSummary, RunError> {
    let mut child = Command::new("nixos-rebuild")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunError::Spawn)?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RunError::Internal("failed to capture child stdout".to_owned()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RunError::Internal("failed to capture child stderr".to_owned()))?;

    let (sender, receiver) = mpsc::channel();
    let stdout_sender = sender.clone();

    let stdout_handle =
        thread::spawn(move || read_stream(StreamKind::Stdout, stdout, stdout_sender));
    let stderr_handle = thread::spawn(move || read_stream(StreamKind::Stderr, stderr, sender));

    let renderer = Renderer::new();
    let mut analyzer = Analyzer::new();

    for event in receiver {
        let kind = analyzer.observe_line(event.stream, &event.line);
        renderer
            .print_live_line(event.stream, kind, &event.line)
            .map_err(RunError::Io)?;
    }

    join_reader(stdout_handle)?;
    join_reader(stderr_handle)?;

    let status = child.wait().map_err(RunError::Spawn)?;
    let summary = analyzer.finish(status.code().unwrap_or(1));
    renderer.print_summary(&summary).map_err(RunError::Io)?;

    Ok(summary)
}

#[derive(Debug)]
enum RunError {
    Spawn(io::Error),
    Io(io::Error),
    Reader(io::Error),
    ThreadPanic,
    Internal(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(error) => write!(f, "failed to start nixos-rebuild: {error}"),
            Self::Io(error) => write!(f, "I/O error while handling nixos-rebuild output: {error}"),
            Self::Reader(error) => write!(f, "failed while reading nixos-rebuild output: {error}"),
            Self::ThreadPanic => write!(f, "output reader thread panicked"),
            Self::Internal(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for RunError {}

struct LineEvent {
    stream: StreamKind,
    line: String,
}

fn read_stream<R: io::Read>(
    stream: StreamKind,
    reader: R,
    sender: mpsc::Sender<LineEvent>,
) -> io::Result<()> {
    let buffered = BufReader::new(reader);

    for line in buffered.lines() {
        let line = line?;
        if sender.send(LineEvent { stream, line }).is_err() {
            break;
        }
    }

    Ok(())
}

fn join_reader(handle: thread::JoinHandle<io::Result<()>>) -> Result<(), RunError> {
    match handle.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(RunError::Reader(error)),
        Err(_) => Err(RunError::ThreadPanic),
    }
}

#[cfg(test)]
mod tests {
    use super::{Analyzer, Renderer, RunSummary, Status, StreamKind};

    #[test]
    fn successful_run_without_markers_is_no_changes() {
        let mut analyzer = Analyzer::new();
        analyzer.observe_line(StreamKind::Stdout, "building the system configuration...");

        let summary = analyzer.finish(0);

        assert_eq!(summary.status, Status::NoChanges);
        assert!(!summary.saw_change);
        assert!(!summary.saw_failure_hint);
        assert!(summary.highlights.is_empty());
    }

    #[test]
    fn would_restart_marker_is_ok() {
        let mut analyzer = Analyzer::new();
        analyzer.observe_line(
            StreamKind::Stderr,
            "would restart the following units: sshd.service",
        );

        let summary = analyzer.finish(0);

        assert_eq!(summary.status, Status::Ok);
        assert!(summary.saw_change);
        assert_eq!(
            summary.highlights,
            vec!["would restart the following units: sshd.service"]
        );
    }

    #[test]
    fn activating_configuration_marker_is_ok() {
        let mut analyzer = Analyzer::new();
        analyzer.observe_line(StreamKind::Stderr, "activating the configuration...");

        let summary = analyzer.finish(0);

        assert_eq!(summary.status, Status::Ok);
        assert!(summary.saw_change);
    }

    #[test]
    fn error_hint_forces_failure_even_with_zero_exit() {
        let mut analyzer = Analyzer::new();
        analyzer.observe_line(
            StreamKind::Stderr,
            "error: flake does not provide attribute",
        );

        let summary = analyzer.finish(0);

        assert_eq!(summary.status, Status::Failure);
        assert!(summary.saw_failure_hint);
    }

    #[test]
    fn nonzero_exit_forces_failure() {
        let mut analyzer = Analyzer::new();
        analyzer.observe_line(StreamKind::Stdout, "plain output");

        let summary = analyzer.finish(1);

        assert_eq!(summary.status, Status::Failure);
        assert!(!summary.saw_failure_hint);
    }

    #[test]
    fn failed_units_warning_is_failure() {
        let mut analyzer = Analyzer::new();
        analyzer.observe_line(
            StreamKind::Stderr,
            "warning: the following units failed: nginx.service",
        );

        let summary = analyzer.finish(0);

        assert_eq!(summary.status, Status::Failure);
        assert_eq!(
            summary.highlights,
            vec!["warning: the following units failed: nginx.service"]
        );
    }

    #[test]
    fn duplicate_highlights_are_deduplicated() {
        let mut analyzer = Analyzer::new();
        for _ in 0..3 {
            analyzer.observe_line(StreamKind::Stderr, "error: duplicate");
        }

        let summary = analyzer.finish(0);

        assert_eq!(summary.highlights, vec!["error: duplicate"]);
    }

    #[test]
    fn mixed_streams_still_produce_correct_status() {
        let mut analyzer = Analyzer::new();
        analyzer.observe_line(StreamKind::Stdout, "building...");
        analyzer.observe_line(
            StreamKind::Stderr,
            "would start the following units: tailscaled.service",
        );
        analyzer.observe_line(StreamKind::Stderr, "done");

        let summary = analyzer.finish(0);

        assert_eq!(summary.status, Status::Ok);
        assert!(summary.saw_change);
        assert!(!summary.saw_failure_hint);
    }

    #[test]
    fn no_changes_summary_mentions_missing_markers() {
        let summary = RunSummary {
            status: Status::NoChanges,
            exit_code: 0,
            saw_change: false,
            saw_failure_hint: false,
            highlights: Vec::new(),
        };

        let rendered = Renderer::plain().render_summary(&summary);

        assert!(rendered.contains("Status: NO CHANGES"));
        assert!(rendered.contains("- no change markers were observed"));
    }

    #[test]
    fn summary_lists_highlights() {
        let summary = RunSummary {
            status: Status::Failure,
            exit_code: 1,
            saw_change: false,
            saw_failure_hint: true,
            highlights: vec!["error: boom".to_owned()],
        };

        let rendered = Renderer::plain().render_summary(&summary);

        assert!(rendered.contains("Status: FAILURE"));
        assert!(rendered.contains("Exit code: 1"));
        assert!(rendered.contains("- error: boom"));
    }
}
