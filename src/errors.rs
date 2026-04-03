use anyhow::Error;

#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    Generic = 1,
    Config = 2,
    Auth = 3,
    Permission = 4,
    NotFound = 5,
    Validation = 6,
    Network = 7,
}

pub fn infer_exit_code(error: &Error) -> i32 {
    let message = format!("{error:#}").to_lowercase();

    if message.contains("apikey")
        || message.contains("authentication")
        || message.contains("unauthorized")
    {
        return ExitCode::Auth as i32;
    }

    if message.contains("config") || message.contains("baseurl") || message.contains("workspace") {
        return ExitCode::Config as i32;
    }

    if message.contains("approval")
        || message.contains("blocked")
        || message.contains("outside workspace")
    {
        return ExitCode::Permission as i32;
    }

    if message.contains("not found") {
        return ExitCode::NotFound as i32;
    }

    if message.contains("required")
        || message.contains("invalid")
        || message.contains("unsupported")
    {
        return ExitCode::Validation as i32;
    }

    if message.contains("timeout")
        || message.contains("connection")
        || message.contains("request failed")
    {
        return ExitCode::Network as i32;
    }

    ExitCode::Generic as i32
}
