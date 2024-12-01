use std::process::ExitCode;

use backlight_control_rs::*;
use clap::{Parser, ValueHint};
use regex::Regex;

fn value_validator(value: &str) -> std::result::Result<String, String> {
    if value.is_empty() {
        return Ok("".to_string());
    }

    let re = Regex::new(r"^[+-]?[0-9]+%?$").unwrap();

    if re.is_match(value) {
        Ok(value.to_string())
    } else {
        Err("Value provided does not match proper form.".to_string())
    }
}

#[derive(Parser, Debug)]
#[command(
    version,
    arg_required_else_help = true,
    about = "backlight_control_rs | a simple util for controlling the backlight brightness on your device"
)]
struct Args {
    /// The value to set / adjust the brightness by
    ///
    /// Examples: +50 | -10 | 200 | 50% | +10%
    #[arg(value_hint = ValueHint::Other, value_parser = value_validator, allow_hyphen_values = true)]
    value: Option<String>,
    /// Print backlight information
    ///
    /// If this is used no value will be set, even if provided
    #[arg(short, long)]
    stats: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.stats {
        let max_brightness = get_max_brightness();
        let brightness = get_brightness();

        let exit_code = if max_brightness.is_ok() && brightness.is_ok() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };

        let max_brightness_output = max_brightness
            .as_ref()
            .map(u32::to_string)
            .unwrap_or_else(|e| format!("Failed to get max brightness: {}", e));
        let brightness_output = brightness
            .as_ref()
            .map(u32::to_string)
            .unwrap_or_else(|e| format!("Failed to get brightness: {}", e));

        println!("Max: {}", max_brightness_output);
        println!("Current: {}", brightness_output);
        return exit_code;
    }

    let value = args.value.unwrap();

    let is_percentage = value.ends_with("%");
    let value_string = value
        .chars()
        .take(value.len() - is_percentage as usize)
        .collect::<String>();

    // NOTE: Unwrapping here *should* be safe since we know the string has len of at least 1
    let result = match value.chars().next().unwrap() {
        '+' | '-' => {
            let parse_result: i32 = value_string.parse().unwrap();
            adjust_brightness_relative(parse_result, is_percentage)
        }
        _ => {
            let parse_result: u32 = value_string.parse().unwrap();
            adjust_brightness_absolute(parse_result, is_percentage)
        }
    };

    if let Err(e) = result {
        eprintln!("Failed to adjust brightness: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
