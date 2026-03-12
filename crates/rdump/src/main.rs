use rdump::run;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            let classified = rdump::request::classify_error_message(&err.to_string());
            match classified.code {
                rdump::contracts::ErrorCode::QuerySyntax
                | rdump::contracts::ErrorCode::QueryValidation
                | rdump::contracts::ErrorCode::InvalidRequest => ExitCode::from(2),
                _ => ExitCode::from(1),
            }
        }
    }
}
