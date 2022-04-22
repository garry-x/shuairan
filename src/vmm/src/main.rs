//
// Copyright 2022 Garry Xu
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod config;

use config::VMConfig;

/// Exit codes returned by the executable.
/// Refers to: https://tldp.org/LDP/abs/html/exitcodes.html
#[derive(Debug, PartialEq)]
enum ExitCode {
    /// Everything goes well
    Ok = 0,
    /// Error coccurs and the error message should be checked.
    GeneralError = 1,
}

fn usage() {
    print!("ShuaiRan v{}\n", env!("CARGO_PKG_VERSION"));
    print!("Usage:\n");
    print!("./shuairan <config>     Start a vm with the given config file.\n");
}

fn vmm_main(path: &str) -> ExitCode {
    // Convert JSON config to VMConfig 
    match VMConfig::from_file(path) {
        Ok(config) => {
            println!("{:?}", config);
            ExitCode::Ok
        },
        _ => ExitCode::GeneralError
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        usage();
        return;
    }
    std::process::exit(vmm_main(&args[1]) as i32);
}
