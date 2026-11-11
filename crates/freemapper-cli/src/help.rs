//! `--help` and the parse errors in the language of the run (ADR 0092, stage 4).
//!
//! The doc comments of the derive stay the English help; the catalog holds the same text under
//! `help-…` ids, and a test compares the two. [`localized`] takes a command built by the derive
//! and replaces every text with the catalog's, clap's own words included (headings, "possible
//! values", "default"). The usage line keeps clap's placeholders (`[OPTIONS]`, `<COMMAND>`).
//!
//! Ids: `help-<command>` for a command (`help-freemapper` is the program), `help-<command>-<arg>`
//! for an argument, and `help-arg-<arg>` for an argument that reads the same in every command.

use std::ffi::OsString;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, Command};

use crate::text::{FluentValue, Lang, TEXTS};

/// The `--lang` value in the arguments: `--lang de` or `--lang=de`, the last one wins. The scan
/// runs before clap parses, because the help text itself needs the language.
pub fn lang_from_args(args: &[OsString]) -> Option<String> {
    let mut found = None;
    let mut iter = args.iter().skip(1).map(|a| a.to_string_lossy());
    while let Some(arg) = iter.next() {
        if arg == "--" {
            break;
        }
        if arg == "--lang" {
            found = iter.next().map(|v| v.into_owned());
        } else if let Some(value) = arg.strip_prefix("--lang=") {
            found = Some(value.to_owned());
        }
    }
    found
}

/// The id of the command reached by `path` (the names below the program).
fn command_id(path: &[String]) -> String {
    if path.is_empty() {
        "help-freemapper".to_owned()
    } else {
        format!("help-{}", path.join("-"))
    }
}

/// The ids an argument's text is looked up under, the command's own first.
fn arg_ids(path: &[String], arg: &str) -> [String; 2] {
    let arg = arg.replace('_', "-");
    [
        format!("{}-{arg}", command_id(path)),
        format!("help-arg-{arg}"),
    ]
}

/// The text of the first of `ids` the catalog has, in `lang`.
fn lookup(lang: &Lang, ids: &[String]) -> Option<String> {
    let english = TEXTS.message_ids("en");
    let id = ids.iter().find(|id| english.contains(*id))?;
    Some(TEXTS.text(lang, id, &[]))
}

/// The short form of a help text as the derive makes it: the first paragraph without its
/// final full stop.
fn short(full: &str) -> String {
    let first = full.split("\n\n").next().unwrap_or(full);
    first.strip_suffix('.').unwrap_or(first).to_owned()
}

/// `cmd`, built by the derive, with every text from the catalog in `lang`.
pub fn localized(cmd: Command, lang: &Lang) -> Command {
    let t = |id: &str| TEXTS.text(lang, id, &[]);
    let help_flag = Arg::new("help")
        .short('h')
        .long("help")
        .action(clap::ArgAction::Help)
        .global(true)
        .help(t("cli-arg-help"));
    let version_flag = Arg::new("version")
        .short('V')
        .long("version")
        .action(clap::ArgAction::Version)
        .help(t("cli-arg-version"));
    let help_command = Command::new("help").about(t("cli-help-help")).arg(
        Arg::new("command")
            .num_args(0..)
            .help(t("cli-arg-help-command")),
    );
    let cmd = cmd
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(help_flag)
        .arg(version_flag)
        .subcommand(help_command);
    localize_tree(cmd, lang, &[])
}

fn localize_tree(cmd: Command, lang: &Lang, path: &[String]) -> Command {
    let t = |id: &str| TEXTS.text(lang, id, &[]);
    let template = format!(
        "{{about-with-newline}}\n{} {{usage}}\n\n{{all-args}}{{after-help}}",
        t("cli-heading-usage")
    );
    // Clap wants 'static headings; the program builds its command once, so leaking is harmless.
    let leak = |id: &str| -> &'static str {
        Box::leak(t(id).trim_end_matches(':').to_owned().into_boxed_str())
    };
    let (options, arguments, commands) = (
        leak("cli-heading-options"),
        leak("cli-heading-arguments"),
        leak("cli-heading-commands"),
    );
    let (possible, default) = (t("cli-help-possible-values"), t("cli-help-default"));

    let mut cmd = cmd;
    if let Some(full) = lookup(lang, &[command_id(path)]) {
        let long = cmd.get_long_about().is_some();
        cmd = cmd.about(short(&full));
        if long {
            cmd = cmd.long_about(full);
        }
    }
    // Clap sorts the help by display order and then by name, so the arguments get their
    // declaration order here; global arguments (--lang, --help) come after a command's own.
    let mut order = 0;
    cmd = cmd
        .help_template(template)
        .subcommand_help_heading(commands)
        .mut_args(|arg| {
            order += 1;
            let offset = if arg.is_global_set() { 1000 } else { 0 };
            let mut arg = arg.display_order(offset + order);
            if let Some(full) = lookup(lang, &arg_ids(path, arg.get_id().as_str())) {
                let long = arg.get_long_help().is_some();
                arg = arg.help(short(&full));
                if long {
                    arg = arg.long_help(full);
                }
            }
            // Clap's "[default: …]" and "[possible values: …]" are English; the catalog's
            // words replace them, in the short and the long help.
            let takes_values = arg.get_action().takes_values();
            let values: Vec<String> = arg
                .get_possible_values()
                .iter()
                .filter(|_| takes_values)
                .filter(|v| !v.is_hide_set())
                .map(|v| v.get_name().to_owned())
                .collect();
            let defaults: Vec<String> = arg
                .get_default_values()
                .iter()
                .filter(|_| takes_values && !arg.is_hide_default_value_set())
                .map(|v| v.to_string_lossy().into_owned())
                .collect();
            let mut notes = String::new();
            if !defaults.is_empty() {
                notes.push_str(&format!(" [{default}: {}]", defaults.join(", ")));
            }
            if !values.is_empty() {
                notes.push_str(&format!(" [{possible}: {}]", values.join(", ")));
            }
            if !notes.is_empty() {
                let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
                let long_help = arg.get_long_help().map(ToString::to_string);
                arg = arg
                    .hide_possible_values(true)
                    .hide_default_value(true)
                    .help(format!("{help}{notes}"));
                if let Some(long_help) = long_help {
                    arg = arg.long_help(format!("{long_help}\n\n{}", notes.trim_start()));
                }
            }
            if arg.is_positional() {
                arg.help_heading(arguments)
            } else {
                arg.help_heading(options)
            }
        });
    let names: Vec<String> = cmd
        .get_subcommands()
        .map(|c| c.get_name().to_owned())
        .collect();
    for name in names {
        let mut below = path.to_vec();
        below.push(name.clone());
        cmd = cmd.mut_subcommand(&name, |sub| localize_tree(sub, lang, &below));
    }
    cmd
}

/// Text of a context value (an argument name, a value, a list of them).
fn context(error: &clap::Error, kind: ContextKind) -> String {
    match error.get(kind) {
        Some(ContextValue::String(s)) => s.clone(),
        Some(ContextValue::Strings(list)) => list.join(", "),
        Some(ContextValue::StyledStr(s)) => s.to_string(),
        Some(ContextValue::StyledStrs(list)) => list
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        Some(ContextValue::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// The catalog text for the common parse errors; `None` for the rare kinds, which keep clap's
/// English text.
pub fn parse_error_message(lang: &Lang, error: &clap::Error) -> Option<String> {
    let arg = || FluentValue::from(context(error, ContextKind::InvalidArg));
    let value = || FluentValue::from(context(error, ContextKind::InvalidValue));
    let (id, args): (&str, Vec<(&str, FluentValue)>) = match error.kind() {
        ErrorKind::UnknownArgument => ("cli-parse-unknown-argument", vec![("arg", arg())]),
        ErrorKind::MissingRequiredArgument => ("cli-parse-missing-required", vec![("args", arg())]),
        ErrorKind::InvalidValue => match context(error, ContextKind::ValidValue) {
            values if values.is_empty() => (
                "cli-parse-invalid-value",
                vec![("arg", arg()), ("value", value())],
            ),
            values => (
                "cli-parse-invalid-value-choices",
                vec![
                    ("arg", arg()),
                    ("value", value()),
                    ("values", FluentValue::from(values)),
                ],
            ),
        },
        ErrorKind::InvalidSubcommand => {
            // Clap stores the unknown word under its own key.
            let word = match context(error, ContextKind::InvalidSubcommand) {
                word if word.is_empty() => context(error, ContextKind::InvalidArg),
                word => word,
            };
            (
                "cli-parse-invalid-subcommand",
                vec![("value", FluentValue::from(word))],
            )
        }
        ErrorKind::MissingSubcommand => ("cli-parse-missing-subcommand", vec![]),
        ErrorKind::TooManyValues => (
            "cli-parse-too-many-values",
            vec![("arg", arg()), ("value", value())],
        ),
        ErrorKind::ArgumentConflict => (
            "cli-parse-argument-conflict",
            vec![
                ("arg", arg()),
                (
                    "other",
                    FluentValue::from(context(error, ContextKind::PriorArg)),
                ),
            ],
        ),
        _ => return None,
    };
    Some(TEXTS.text(lang, id, &args))
}

/// The help of the command `path` names below `root` (`help run`), or `None` when a word of
/// `path` names no command; then the word comes back.
pub fn help_of(root: &mut Command, path: &[String]) -> Result<String, String> {
    root.build();
    let mut found: &Command = root;
    for name in path {
        found = found.find_subcommand(name).ok_or_else(|| name.clone())?;
    }
    Ok(found.clone().render_help().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory as _;
    use std::collections::BTreeSet;

    fn de() -> Lang {
        TEXTS.parse_lang_arg("de").unwrap()
    }

    /// Every command and argument of the derive with its help text: the id it is looked up
    /// under first, the fallback id, and the full text.
    fn derived(cmd: &Command, path: &[String], out: &mut Vec<([String; 2], String)>) {
        if let Some(about) = cmd.get_long_about().or(cmd.get_about()) {
            out.push(([command_id(path), command_id(path)], about.to_string()));
        }
        for arg in cmd.get_arguments() {
            if let Some(help) = arg.get_long_help().or(arg.get_help()) {
                out.push((arg_ids(path, arg.get_id().as_str()), help.to_string()));
            }
        }
        for sub in cmd.get_subcommands() {
            let mut below = path.to_vec();
            below.push(sub.get_name().to_owned());
            derived(sub, &below, out);
        }
    }

    fn all_derived() -> Vec<([String; 2], String)> {
        let mut out = Vec::new();
        derived(&crate::Cli::command(), &[], &mut out);
        out
    }

    #[test]
    fn the_english_help_equals_the_doc_comments() {
        let english = TEXTS.message_ids("en");
        for (ids, text) in all_derived() {
            let id = ids
                .iter()
                .find(|id| english.contains(*id))
                .unwrap_or_else(|| panic!("no help text for {} in the catalog", ids[0]));
            assert_eq!(TEXTS.text(&Lang::english(), id, &[]), text, "{id}");
        }
    }

    #[test]
    fn every_help_text_belongs_to_a_command_or_an_argument() {
        let reachable: BTreeSet<String> =
            all_derived().into_iter().flat_map(|(ids, _)| ids).collect();
        for id in TEXTS.message_ids("en") {
            if id.starts_with("help-") {
                assert!(reachable.contains(&id), "{id} names no command or argument");
            }
        }
    }

    #[test]
    fn the_localized_commands_are_valid_in_every_language() {
        for lang in TEXTS.available() {
            localized(crate::Cli::command(), &lang).debug_assert();
        }
    }

    #[test]
    fn the_german_help_has_german_headings_and_texts() {
        let mut cmd = localized(crate::Cli::command(), &de());
        let help = help_of(&mut cmd, &["init".to_owned()]).unwrap();
        assert!(help.contains("Aufruf:"), "{help}");
        assert!(help.contains("Optionen:"), "{help}");
        assert!(help.contains("[Standard: schoolyard]"), "{help}");
        assert!(
            help.contains("[mögliche Werte: schoolyard, sprint]"),
            "{help}"
        );
        assert!(!help.contains("possible values"), "{help}");
        assert!(!help.contains("Usage:"), "{help}");
        assert_eq!(
            help_of(&mut cmd, &["nope".to_owned()]),
            Err("nope".to_owned())
        );
    }

    #[test]
    fn the_lang_argument_is_found_before_parsing() {
        let args = |list: &[&str]| -> Vec<OsString> { list.iter().map(OsString::from).collect() };
        assert_eq!(lang_from_args(&args(&["freemapper", "--help"])), None);
        assert_eq!(
            lang_from_args(&args(&["freemapper", "--lang", "de", "--help"])),
            Some("de".to_owned())
        );
        assert_eq!(
            lang_from_args(&args(&["freemapper", "run", "--lang=en", "--lang=de"])),
            Some("de".to_owned())
        );
        assert_eq!(
            lang_from_args(&args(&["freemapper", "--", "--lang", "de"])),
            None
        );
    }

    /// Prints the English catalog entries from the doc comments: `cargo test -p freemapper-cli
    /// print_the_english_help -- --ignored --nocapture`. An argument that reads the same in
    /// every command it has gets one `help-arg-…` entry.
    #[test]
    #[ignore = "a tool for the catalog, not a test"]
    fn print_the_english_help() {
        let all = all_derived();
        let mut shared: std::collections::BTreeMap<String, BTreeSet<String>> = Default::default();
        let mut uses: std::collections::BTreeMap<String, usize> = Default::default();
        for (ids, text) in &all {
            if ids[0] != ids[1] {
                shared
                    .entry(ids[1].clone())
                    .or_default()
                    .insert(text.clone());
                *uses.entry(ids[1].clone()).or_default() += 1;
            }
        }
        let mut printed = BTreeSet::new();
        for (ids, text) in &all {
            let one_text = shared.get(&ids[1]).is_some_and(|texts| texts.len() == 1);
            let id = if ids[0] != ids[1] && one_text && uses[&ids[1]] > 1 {
                &ids[1]
            } else {
                &ids[0]
            };
            if !printed.insert(id.clone()) {
                continue;
            }
            let escaped = text
                .replace('{', "\u{1}")
                .replace('}', "{\"}\"}")
                .replace('\u{1}', "{\"{\"}");
            if escaped.contains('\n') {
                println!("{id} =");
                for line in escaped.lines() {
                    println!("    {line}");
                }
            } else {
                println!("{id} = {escaped}");
            }
        }
    }
}
