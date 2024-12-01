//! This lib allows for simple adjustment of the users backlight.
use std::{
    cmp::{max, min},
    fs,
    path::PathBuf,
};

pub mod error;

pub use error::*;

const BRIGHTNESS_BASE_PATH: &str = "/sys/class/backlight";

/// Internal function to get the path of the backlight
///
/// This is different from system to system, since it depends on the GPU.
fn vendor_path() -> Result<PathBuf> {
    let vendor_dir = fs::read_dir(BRIGHTNESS_BASE_PATH)?
        .filter_map(std::io::Result::ok) // Filter out any errors
        .next() // Get the first entry
        .ok_or(Error::FailedToGetFirstEntry)?
        .file_name(); // Get the file name directly

    Ok(PathBuf::from(BRIGHTNESS_BASE_PATH).join(vendor_dir))
}

/// Gets the max brightness
///
/// ## Panics
///
/// Will only panic if it cannot parse the value in in the `max_brightness` file. This should never
/// happen. If it does this is an OS error.
pub fn get_max_brightness() -> Result<u32> {
    let max_brightness_path = vendor_path()?.join("max_brightness");

    Ok(fs::read_to_string(&max_brightness_path)?
        .trim()
        .parse()
        .expect("Failed to parse max brightness. This should never happen."))
}

/// Gets the current brightness
///
/// ## Panics
///
/// Will only panic if it cannot parse the value in in the `brightness` file. This should never
/// happen. If it does this is an OS error.
pub fn get_brightness() -> Result<u32> {
    let brightness_path = vendor_path()?.join("brightness");

    Ok(fs::read_to_string(&brightness_path)?
        .trim()
        .parse()
        .expect("Failed to parse brightness. This should never happen."))
}

/// Attempts to set the brightness
///
/// This will fail if you lack the proper permissions.
pub fn set_brightness(value: u32) -> Result<()> {
    let brightness_path = vendor_path()?.join("brightness");

    fs::write(
        brightness_path,
        min(value, get_max_brightness()?).to_string(),
    )?;

    Ok(())
}

/// Convenience wrapper around set_brightness for adjusting relative to the current value
///
/// # Parameters
///
/// - `value`: The value to adjust by.
///
/// - `percentage`: Will treat the value as a percentage to adjust by, rather than an absolute value.
pub fn adjust_brightness_relative(value: i32, percentage: bool) -> Result<()> {
    let brightness: i32 = get_brightness()?.try_into().unwrap();

    if percentage {
        let max_brightness = get_max_brightness()?;

        let new_brightness = (max_brightness as i32 * value / 100 + brightness).max(0) as u32;

        set_brightness(new_brightness)?;

        return Ok(());
    }

    let new_brightness = brightness + value;

    set_brightness(max(0, new_brightness).try_into().unwrap())
}

/// Convenience wrapper around set_brightness for setting absolute values
///
/// ## Parameters
///
/// - `value`: The value to set to.
///
/// - `percentage`: Will treat the value as a percentage of the max to set to, rather than an absolute value.
pub fn adjust_brightness_absolute(value: u32, percentage: bool) -> Result<()> {
    if percentage {
        let max_brightness = get_max_brightness()?;

        let new_brightness = max_brightness * value / 100;

        set_brightness(new_brightness)?;

        return Ok(());
    }

    set_brightness(value)
}
