#[derive(serde::Serialize, thiserror::Error, specta::Type, Debug)]
#[error(
    r#"
------tauri command failed------
Root cause: {root_cause}

{msg}
--------------------------------
"#
)]
pub struct AnyError {
    root_cause: String,
    msg: String,
}

impl From<eyre::Report> for AnyError {
    fn from(value: eyre::Report) -> Self {
        let root_cause = value.root_cause();

        Self {
            root_cause: format!("{}", root_cause),
            msg: format_eyre_message(value),
        }
    }
}

fn format_eyre_message(report: eyre::Report) -> String {
    let msg = format!("{:?}", report);
    let mut lines: Vec<&str> = msg.lines().collect();

    while let Some(last_line) = lines.last() {
        if last_line.starts_with("Backtrace omitted.")
            || last_line.starts_with("Run with RUST_BACKTRACE")
        {
            lines.pop();
        } else {
            break;
        }
    }

    lines.join("\n")
}
