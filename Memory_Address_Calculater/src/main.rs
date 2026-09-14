//! Memory Address Calculator
//!
//! Converts memory addresses from reference (ROM) space to working (RAM) space.
//! For internal company use.

use std::fs;
use std::io::{self, Write};
use std::process::Command;

/// Configuration containing base addresses for conversion
#[derive(Debug)]
struct Config {
    reference_base: u64,
    working_base: u64,
}

/// Result of a successful address conversion
#[derive(Debug)]
struct ConversionResult {
    input_address: u64,
    reference_base: u64,
    working_base: u64,
    offset: u64,
    converted_address: u64,
}

/// Error types for configuration loading
#[derive(Debug)]
enum ConfigError {
    FileNotFound(String),
    ReadError(String),
    MissingKey(String),
    InvalidValue { key: String, value: String },
}

/// Error types for address conversion
#[derive(Debug)]
enum ConversionError {
    EmptyInput,
    InvalidHexFormat(String),
    AddressBelowBase { input: u64, base: u64 },
    Overflow,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => {
                write!(f, "Config file not found: {}", path)
            }
            ConfigError::ReadError(msg) => {
                write!(f, "Failed to read config file: {}", msg)
            }
            ConfigError::MissingKey(key) => {
                write!(f, "Missing required config key: {}", key)
            }
            ConfigError::InvalidValue { key, value } => {
                write!(f, "Invalid value for '{}': '{}'", key, value)
            }
        }
    }
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::EmptyInput => {
                write!(f, "Empty input. Please enter an address.")
            }
            ConversionError::InvalidHexFormat(input) => {
                write!(f, "Invalid hexadecimal format: '{}'", input)
            }
            ConversionError::AddressBelowBase { input, base } => {
                write!(
                    f,
                    "Input address {} is below reference base {}",
                    format_address(*input),
                    format_address(*base)
                )
            }
            ConversionError::Overflow => {
                write!(f, "Arithmetic overflow during conversion")
            }
        }
    }
}

/// Loads configuration from the specified file path
fn load_config(path: &str) -> Result<String, ConfigError> {
    fs::read_to_string(path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            ConfigError::FileNotFound(path.to_string())
        } else {
            ConfigError::ReadError(e.to_string())
        }
    })
}

/// Parses a hexadecimal string value from config
/// Accepts formats: 0xABCD, 0XABCD, ABCD
fn parse_hex_value(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    let hex_str = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        &trimmed[2..]
    } else {
        trimmed
    };

    if hex_str.is_empty() {
        return None;
    }

    u64::from_str_radix(hex_str, 16).ok()
}

/// Parses the configuration content into a Config struct
fn parse_config(content: &str) -> Result<Config, ConfigError> {
    let mut reference_base: Option<u64> = None;
    let mut working_base: Option<u64> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "reference_base" => {
                    reference_base = Some(
                        parse_hex_value(value).ok_or_else(|| ConfigError::InvalidValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        })?,
                    );
                }
                "working_base" => {
                    working_base = Some(
                        parse_hex_value(value).ok_or_else(|| ConfigError::InvalidValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        })?,
                    );
                }
                _ => {
                    // Ignore unknown keys for forward compatibility
                }
            }
        }
    }

    let reference_base =
        reference_base.ok_or_else(|| ConfigError::MissingKey("reference_base".to_string()))?;

    let working_base =
        working_base.ok_or_else(|| ConfigError::MissingKey("working_base".to_string()))?;

    Ok(Config {
        reference_base,
        working_base,
    })
}

/// Parses user input address
/// Accepts formats: 0xABCD, 0XABCD, ABCD (all treated as hexadecimal)
fn parse_address(input: &str) -> Result<u64, ConversionError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(ConversionError::EmptyInput);
    }

    // Remove 0x or 0X prefix if present
    let hex_str = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        &trimmed[2..]
    } else {
        trimmed
    };

    if hex_str.is_empty() {
        return Err(ConversionError::InvalidHexFormat(input.to_string()));
    }

    // Validate that all characters are valid hex digits
    if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ConversionError::InvalidHexFormat(input.to_string()));
    }

    u64::from_str_radix(hex_str, 16)
        .map_err(|_| ConversionError::InvalidHexFormat(input.to_string()))
}

/// Performs the address conversion calculation
fn convert_address(input_address: u64, config: &Config) -> Result<ConversionResult, ConversionError> {
    // Check if input address is below reference base
    if input_address < config.reference_base {
        return Err(ConversionError::AddressBelowBase {
            input: input_address,
            base: config.reference_base,
        });
    }

    // Calculate offset with overflow check
    let offset = input_address
        .checked_sub(config.reference_base)
        .ok_or(ConversionError::Overflow)?;

    // Calculate converted address with overflow check
    let converted_address = config
        .working_base
        .checked_add(offset)
        .ok_or(ConversionError::Overflow)?;

    Ok(ConversionResult {
        input_address,
        reference_base: config.reference_base,
        working_base: config.working_base,
        offset,
        converted_address,
    })
}

/// Formats a u64 address as uppercase hexadecimal with 0x prefix
fn format_address(address: u64) -> String {
    format!("0x{:08X}", address)
}

/// Formats an offset value (may have different padding for readability)
fn format_offset(offset: u64) -> String {
    format!("0x{:08X}", offset)
}

/// Prints the conversion result in a readable format
fn print_result(result: &ConversionResult) {
    println!();
    println!(
        "Input Address       : {}",
        format_address(result.input_address)
    );
    println!(
        "Reference Base      : {}",
        format_address(result.reference_base)
    );
    println!(
        "Working Base        : {}",
        format_address(result.working_base)
    );
    println!("Offset              : {}", format_offset(result.offset));
    println!(
        "Converted Address   : {}",
        format_address(result.converted_address)
    );
    println!("RESULT={}", format_address(result.converted_address));
}

/// Attempts to copy text to clipboard using platform-specific commands.
///
/// Implementation notes:
/// - Windows: Uses `cmd /C echo <text> | clip`
/// - macOS: Uses `echo <text> | pbcopy`
/// - Linux: Attempts `xclip` or `xsel` if available
///
/// This is inherently environment-dependent. The clipboard commands may not be
/// available on all systems (e.g., headless Linux servers, minimal containers).
/// This function fails gracefully and never panics.
fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, use PowerShell's Set-Clipboard for reliable Unicode handling
        // Fallback to cmd /C echo | clip if PowerShell fails
        let ps_result = Command::new("powershell")
            .args(["-Command", &format!("Set-Clipboard -Value '{}'", text)])
            .output();

        match ps_result {
            Ok(output) if output.status.success() => return Ok(()),
            _ => {
                // Fallback to cmd /C echo | clip
                let cmd_result = Command::new("cmd")
                    .args(["/C", &format!("echo {} | clip", text)])
                    .output();

                match cmd_result {
                    Ok(output) if output.status.success() => Ok(()),
                    Ok(output) => Err(format!(
                        "clip command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )),
                    Err(e) => Err(format!("Failed to execute clip: {}", e)),
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use pbcopy via stdin
        use std::process::Stdio;

        let child = Command::new("pbcopy").stdin(Stdio::piped()).spawn();

        match child {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(e) = stdin.write_all(text.as_bytes()) {
                        return Err(format!("Failed to write to pbcopy: {}", e));
                    }
                }
                match child.wait() {
                    Ok(status) if status.success() => Ok(()),
                    Ok(_) => Err("pbcopy exited with error".to_string()),
                    Err(e) => Err(format!("Failed to wait for pbcopy: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to execute pbcopy: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, try xclip first, then xsel as fallback
        use std::process::Stdio;

        // Try xclip first
        let xclip_result = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn();

        if let Ok(mut child) = xclip_result {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // Try xsel as fallback
        let xsel_result = Command::new("xsel")
            .args(["--clipboard", "--input"])
            .stdin(Stdio::piped())
            .spawn();

        if let Ok(mut child) = xsel_result {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        Err("No clipboard utility available (tried xclip, xsel)".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Clipboard not supported on this platform".to_string())
    }
}

/// Prints the startup banner with loaded configuration
fn print_banner(config: &Config) {
    println!("================================================================================");
    println!("                       Memory Address Calculator                               ");
    println!("================================================================================");
    println!();
    println!("Loaded config:");
    println!(
        "  reference_base : {}",
        format_address(config.reference_base)
    );
    println!("  working_base   : {}", format_address(config.working_base));
    println!();
    println!("Enter 'q' or 'exit' to quit.");
    println!("--------------------------------------------------------------------------------");
}

/// Prompts user for input and returns the trimmed string
fn prompt_input() -> io::Result<String> {
    print!("\nEnter address > ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

/// Checks if the input is an exit command
fn is_exit_command(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower == "q" || lower == "exit"
}

/// Main entry point
fn main() {
    const CONFIG_PATH: &str = "config.txt";

    // Load and parse configuration
    let config = match load_config(CONFIG_PATH) {
        Ok(content) => match parse_config(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("Please check your config.txt file format.");
                eprintln!();
                eprintln!("Expected format:");
                eprintln!("  reference_base=0xA021C000");
                eprintln!("  working_base=0x60100000");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            eprintln!("Please create a config.txt file with the following format:");
            eprintln!("  reference_base=0xA021C000");
            eprintln!("  working_base=0x60100000");
            std::process::exit(1);
        }
    };

    // Print startup banner
    print_banner(&config);

    // Main interactive loop
    loop {
        // Get user input
        let input = match prompt_input() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        };

        // Check for exit commands
        if is_exit_command(&input) {
            println!("\nGoodbye!");
            break;
        }

        // Skip empty input with a gentle reminder
        if input.is_empty() {
            println!("Please enter an address (or 'q' to quit).");
            continue;
        }

        // Parse the address
        let address = match parse_address(&input) {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };

        // Perform conversion
        let result = match convert_address(address, &config) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };

        // Print the result
        print_result(&result);

        // Attempt clipboard copy
        let clipboard_text = format_address(result.converted_address);
        match copy_to_clipboard(&clipboard_text) {
            Ok(()) => {
                println!("Clipboard Copy      : Success");
            }
            Err(e) => {
                println!("Clipboard Copy      : Failed ({})", e);
            }
        }

        println!("--------------------------------------------------------------------------------");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_value() {
        assert_eq!(parse_hex_value("0xA021C000"), Some(0xA021C000));
        assert_eq!(parse_hex_value("0XA021C000"), Some(0xA021C000));
        assert_eq!(parse_hex_value("A021C000"), Some(0xA021C000));
        assert_eq!(parse_hex_value("  0xA021C000  "), Some(0xA021C000));
        assert_eq!(parse_hex_value(""), None);
        assert_eq!(parse_hex_value("0x"), None);
    }

    #[test]
    fn test_parse_address() {
        assert_eq!(parse_address("0xA021DCBC").unwrap(), 0xA021DCBC);
        assert_eq!(parse_address("A021DCBC").unwrap(), 0xA021DCBC);
        assert_eq!(parse_address("0XA021DCBC").unwrap(), 0xA021DCBC);
        assert_eq!(parse_address("  A021DCBC  ").unwrap(), 0xA021DCBC);
        assert_eq!(parse_address("60100000").unwrap(), 0x60100000);

        assert!(parse_address("").is_err());
        assert!(parse_address("0x").is_err());
        assert!(parse_address("GHIJK").is_err());
    }

    #[test]
    fn test_convert_address() {
        let config = Config {
            reference_base: 0xA021C000,
            working_base: 0x60100000,
        };

        let result = convert_address(0xA021DCBC, &config).unwrap();
        assert_eq!(result.offset, 0x1CBC);
        assert_eq!(result.converted_address, 0x60101CBC);
    }

    #[test]
    fn test_convert_address_below_base() {
        let config = Config {
            reference_base: 0xA021C000,
            working_base: 0x60100000,
        };

        assert!(convert_address(0xA0000000, &config).is_err());
    }

    #[test]
    fn test_format_address() {
        assert_eq!(format_address(0x60101CBC), "0x60101CBC");
        assert_eq!(format_address(0), "0x00000000");
        assert_eq!(format_address(0xFFFFFFFF), "0xFFFFFFFF");
    }

    #[test]
    fn test_parse_config() {
        let content = r#"
reference_base=0xA021C000
working_base=0x60100000
"#;
        let config = parse_config(content).unwrap();
        assert_eq!(config.reference_base, 0xA021C000);
        assert_eq!(config.working_base, 0x60100000);
    }

    #[test]
    fn test_parse_config_without_prefix() {
        let content = r#"
reference_base=A021C000
working_base=60100000
"#;
        let config = parse_config(content).unwrap();
        assert_eq!(config.reference_base, 0xA021C000);
        assert_eq!(config.working_base, 0x60100000);
    }

    #[test]
    fn test_is_exit_command() {
        assert!(is_exit_command("q"));
        assert!(is_exit_command("Q"));
        assert!(is_exit_command("exit"));
        assert!(is_exit_command("EXIT"));
        assert!(is_exit_command("Exit"));
        assert!(!is_exit_command("quit"));
        assert!(!is_exit_command("0xABCD"));
    }
}
