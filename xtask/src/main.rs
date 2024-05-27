use std::fs;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context;
use clap::builder::styling;
use clap::builder::Styles;
use clap::Parser;

/// Regenerate help for absh.
#[derive(clap::Parser, Debug)]
struct GenReadme {
    /// Do not regenerate help, just check that it can be generated.
    #[clap(long)]
    check: bool,
}

struct ReadmeMd {
    orignial: String,
    before: String,
    after: String,
}

impl GenReadme {
    fn parse_readme_md() -> anyhow::Result<ReadmeMd> {
        eprintln!("Reading README.md");
        let readme_md = fs::read_to_string("README.md")?;
        let lines = readme_md.lines().collect::<Vec<_>>();
        let start = lines
            .iter()
            .position(|&line| line == "<!-- absh-help:start -->")
            .context("<!-- absh-help:start --> not found in README.md")?;
        let end = lines
            .iter()
            .position(|&line| line == "<!-- absh-help:end -->")
            .context("<!-- absh-help:end --> not found in README.md")?;
        let before = lines[..start + 1]
            .iter()
            .map(|&line| format!("{}\n", line))
            .collect::<String>();
        let after = lines[end..]
            .iter()
            .map(|&line| format!("{}\n", line))
            .collect::<String>();
        Ok(ReadmeMd {
            orignial: readme_md,
            before,
            after,
        })
    }

    fn normalize_help_output(help: &str) -> String {
        help.lines()
            .map(|line| {
                let line = line.trim_end();
                format!("{}\n", line)
            })
            .collect()
    }

    fn run(&self) -> anyhow::Result<()> {
        eprintln!("Running absh --help");
        let out = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("absh")
            .arg("--")
            .arg("--help")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()?;

        if !out.status.success() {
            anyhow::bail!("Failed to run absh --help");
        }

        let ReadmeMd {
            orignial,
            before,
            after,
        } = Self::parse_readme_md()?;
        let help = String::from_utf8(out.stdout)?;
        let help = Self::normalize_help_output(&help);

        let readme_md = format!("{before}```\n{help}```\n{after}");
        if self.check {
            anyhow::ensure!(orignial == readme_md, "README.md is not up to date");
            eprintln!("README.md is up to date");
        } else {
            eprintln!("Writing README.md");
            fs::write("README.md", readme_md)?;
        }
        Ok(())
    }
}

pub(crate) fn clap_styles() -> Styles {
    let heading = styling::AnsiColor::Yellow.on_default().bold();
    Styles::styled()
        .header(heading)
        .usage(heading)
        .literal(styling::AnsiColor::Green.on_default())
        .placeholder(styling::AnsiColor::Cyan.on_default())
}

/// absh build helper.
#[derive(clap::Parser, Debug)]
#[clap(styles = clap_styles())]
enum AbshXTaskOpts {
    GenReadme(GenReadme),
}

fn main() -> anyhow::Result<()> {
    let opts: AbshXTaskOpts = AbshXTaskOpts::parse();
    match opts {
        AbshXTaskOpts::GenReadme(gen_help) => gen_help.run(),
    }
}
