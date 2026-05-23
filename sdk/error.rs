/*
 * Nuva OS
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// ! SDK errorclasstypefixedmeaning

use std::fmt;

/// SDK errortype
#[derive(Debug)]
pub enum SdkError {
 /// IO error
 IoError(String),
 /// parseerror
 ParseError(String),
 /// workmakeemptybetweenfindto
 WorkspaceNotFound(String),
 /// packetfindto
 PackageNotFound(String),
 /// dependencyparseerror
 DependencyError(String),
 /// builderror
 BuildError(String),
 /// configureerror
 ConfigError(String),
 /// networkerror
 NetworkError(String),
 /// debugerror
 DebugError(String),
 /// performanceanalyzeerror
 ProfileError(String),
 /// error
 CommandError(String),
 /// notSupport Operation
 Unsupported(String),
 /// Othererror
 Other(String),
 /// filenotfound
 FileNotFound(String),
 /// executionerror
 ExecutionError(String),
 /// testfailed
 TestFailed(String),
 /// notfounderror
 NotFoundError(String),
 /// invalidargument
 InvalidArgument(String),
 /// invalidstate
 InvalidState(String),
 /// serializationerror
 SerializationError(String),
 /// publisherror
 PublishError(String),
 /// validationerror
 ValidationError(String),
 /// authenticationerror
 AuthenticationError(String),
}

impl fmt::Display for SdkError {
 fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 match self {
 SdkError::IoError(msg) => write!(f, "IO error: {}", msg),
 SdkError::ParseError(msg) => write!(f, "Parse error: {}", msg),
 SdkError::WorkspaceNotFound(path) => write!(f, "Workspace not found: {}", path),
 SdkError::PackageNotFound(name) => write!(f, "Package not found: {}", name),
 SdkError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
 SdkError::BuildError(msg) => write!(f, "Build error: {}", msg),
 SdkError::ConfigError(msg) => write!(f, "Config error: {}", msg),
 SdkError::NetworkError(msg) => write!(f, "Network error: {}", msg),
 SdkError::DebugError(msg) => write!(f, "Debug error: {}", msg),
 SdkError::ProfileError(msg) => write!(f, "Profile error: {}", msg),
 SdkError::CommandError(msg) => write!(f, "Command error: {}", msg),
 SdkError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
 SdkError::Other(msg) => write!(f, "Error: {}", msg),
 SdkError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
 SdkError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
 SdkError::TestFailed(msg) => write!(f, "Test failed: {}", msg),
 SdkError::NotFoundError(msg) => write!(f, "Not found: {}", msg),
 SdkError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
 SdkError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
 SdkError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
 SdkError::PublishError(msg) => write!(f, "Publish error: {}", msg),
 SdkError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
 SdkError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
 }
 }
}

impl std::error::Error for SdkError {}

impl From<std::io::Error> for SdkError {
 fn from(e: std::io::Error) -> Self {
 SdkError::IoError(e.to_string())
 }
}

impl From<toml::de::Error> for SdkError {
 fn from(e: toml::de::Error) -> Self {
 SdkError::ParseError(e.to_string())
 }
}

/// SDK resulttype
pub type SdkResult<T> = Result<T, SdkError>;

/// errorcontext
#[derive(Debug)]
pub struct ErrorContext {
 /// error
 pub error: SdkError,
 /// contextinformation
 pub context: String,
 /// sourcelocation
 pub source: Option<String>,
}

impl ErrorContext {
 pub fn new(error: SdkError, context: impl Into<String>) -> Self {
 Self {
 error,
 context: context.into(),
 source: None,
 }
 }

 pub fn with_source(mut self, source: impl Into<String>) -> Self {
 self.source = Some(source.into());
 self
 }
}

impl fmt::Display for ErrorContext {
 fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 write!(f, "{}: {}", self.context, self.error)?;
 if let Some(ref source) = self.source {
 write!(f, " (at {})", source)?;
 }
 Ok(())
 }
}

/// errorlink
#[derive(Debug, Default)]
pub struct ErrorChain {
 errors: Vec<ErrorContext>,
}

impl ErrorChain {
 pub fn new() -> Self {
 Self::default()
 }

 pub fn push(&mut self, ctx: ErrorContext) {
 self.errors.push(ctx);
 }

 pub fn is_empty(&self) -> bool {
 self.errors.is_empty()
 }

 pub fn len(&self) -> usize {
 self.errors.len()
 }

 pub fn iter(&self) -> impl Iterator<Item = &ErrorContext> {
 self.errors.iter()
 }

 pub fn first(&self) -> Option<&ErrorContext> {
 self.errors.first()
 }

 pub fn last(&self) -> Option<&ErrorContext> {
 self.errors.last()
 }
}

impl fmt::Display for ErrorChain {
 fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 for (i, ctx) in self.errors.iter().enumerate() {
 if i > 0 {
 writeln!(f)?;
 }
 write!(f, "[{}] {}", i + 1, ctx)?;
 }
 Ok(())
 }
}

/// warningtype
#[derive(Debug)]
pub struct SdkWarning {
 pub message: String,
 pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
 Low,
 Medium,
 High,
}

impl fmt::Display for SdkWarning {
 fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 let severity = match self.severity {
 WarningSeverity::Low => "LOW",
 WarningSeverity::Medium => "MEDIUM",
 WarningSeverity::High => "HIGH",
 };
 write!(f, "[{}] {}", severity, self.message)
 }
}