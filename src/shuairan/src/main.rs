// Copyright 2022 Garry Xu
// SPDX-License-Identifier: Apache-2.0

use vmm::config::VmConfig;

/// Exit codes returned by the executable.
/// Refers to: https://tldp.org/LDP/abs/html/exitcodes.html
#[derive(Debug, PartialEq)]
enum ExitCode {
    /// Everything goes well
    Ok = 0,
    /// Error coccurs and the error message should be checked.
    GeneralError = 1,
}

/// Help message for the executable.
fn usage() {
    print!("ShuaiRan v{}\n", env!("CARGO_PKG_VERSION"));
    print!("Usage:\n");
    print!("./shuairan <config>     Start a vm with the given config file.\n");
}

/// The entry point function for the hypervisor.  
/// 
/// A VM will be booted according to the given configuration file. 
/// 
/// # Arguments
/// * 'path': Path to a VM's configuration file.
fn vmm_entry(path: &str) -> ExitCode {
    // Convert JSON config to VMConfig 
    match VmConfig::from_file(path) {
        Ok(config) => {
            println!("{:?}", config);
            ExitCode::Ok
        },
        _ => ExitCode::GeneralError
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let code = if args.len() != 2 {
        usage();
        ExitCode::GeneralError
    } else {
        vmm_entry(&args[1])
    };
    std::process::exit(code as i32);
}
