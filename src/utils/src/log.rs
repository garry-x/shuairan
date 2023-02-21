// Copyright 2022 Garry Xu
// SPDX-License-Identifier: Apache-2.0

use std::fs::{File, OpenOptions};
use std::io::{Write, Result};
use chrono::Local;

/// A label for the logger or the logged messages. 
/// The precedence for each level: Error (Highest) > Warn > Info > Debug (Lowest).
/// 
/// The rules for logging message: 
/// - A logger runs at some `LogLevel` L1.
/// - A messsage needs to be logged and labeled with `LogLevel` L2.
/// - If L2 < L1, the message will be ignored. Elsewise, it will be properly logged.
#[derive(Debug, PartialEq, Clone)]
pub enum LogLevel {
    /// Messages with this label are for debug perpose and can be ignored.
    /// Loggers with this label record all incoming messages. 
    Debug,
    /// Messages with this label provide meaningful information. 
    /// Loggers with this label record messages with `LogLevel` >= Info.
    Info,
    /// Messages with this label indicate something noticeable happens and should be checked. 
    /// Loggers with this label record messages with `LogLevel` >= Warn.
    Warn,
    /// Messages with this label indicate some error happened. 
    /// Loggers with this label record messages with `LogLevel` >= Error. 
    Error,
}

/// Common operations shared by all loggers.
pub trait Logger {
    /// Set `LogLevel` for this logger and can be called at run time.
    fn set_level(&mut self, level: LogLevel);

    /// Record a message at some `LogLevel`.
    fn log(&mut self, level: LogLevel, msg: &str) -> Result<()>;

    /// A wrapper for logging messages at `Debug` level.
    fn debug(&mut self, msg: &str) -> Result<()> {
        self.log(LogLevel::Debug, msg)
    }
    /// A wrapper for logging messages at `Info` level.
    fn info(&mut self, msg: &str) -> Result<()> {
        self.log(LogLevel::Info, msg)
    }
    /// A wrapper for logging messages at `Warn` level.
    fn warn(&mut self, msg: &str) -> Result<()> {
        self.log(LogLevel::Warn, msg)
    }
    /// A wrapper for logging messages at `Error` level.
    fn error(&mut self, msg: &str) -> Result<()> {
        self.log(LogLevel::Error, msg)
    }
}

/// A implementation of `Logger` with file backend. To be noticed, it is not thread safe.
struct FileLogger {
    /// `LogLevel` for this logger.
    level: LogLevel,
    /// Handle for the logging file. 
    file: File
}

impl FileLogger {
    fn new(path: &str, level: LogLevel) -> Result<Self> {
        Ok(FileLogger { 
            level, 
            file: OpenOptions::new().append(true)
                                    .create(true)
                                    .open(path)?
        })
    }
}

impl Logger for FileLogger {
    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    fn log(&mut self, level: LogLevel, msg: &str) -> Result<()> {
        if (level as i32) >= (self.level as i32) {
            self.file.write_all(format!(
                "{} {}\n",
                Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                msg
            ).as_bytes())
        } else {
            Ok(())
        }
    }
}

#[test]
pub fn test_file_log() {
    let path = "../../resources/shuairan.log";    
    // Cleanup 
    std::fs::remove_file(path).unwrap();
    // Log some message
    let mut logger = FileLogger::new(path, LogLevel::Debug).unwrap();
    let msg = "debug message";
    logger.debug(msg).unwrap();
    logger.set_level(LogLevel::Info);
    logger.debug(msg).unwrap();
    let buf = std::fs::read_to_string(path).unwrap();
    assert_eq!(buf.lines().collect::<Vec<&str>>().len(), 1);
    assert_eq!(buf.len() > msg.len(), true);
    assert_eq!(
        &buf.as_bytes()[buf.len() - msg.len() - 1..buf.len() - 1],
        msg.as_bytes()
    )
}
