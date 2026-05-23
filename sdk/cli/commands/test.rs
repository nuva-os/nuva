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

//! Test command

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::TestCommand;
use crate::cli::output;

/// Execute test command
pub fn execute(sdk: &mut NuvaSdk, cmd: TestCommand) -> Result<(), SdkError> {
    output::info("Running tests...");
    
    if let Some(filter) = &cmd.filter {
        output::info(&format!("Filter: {}", filter));
    }
    
    // 1. Build test binaries
    output::info("Building test binaries...");
    let build_cmd = crate::cli::args::BuildCommand {
        release: cmd.release,
        target: cmd.target.clone(),
        features: cmd.features.clone(),
        jobs: None,
        opt_level: None,
        debug_info: false,
    };
    
    // Build in test mode
    let test_build = crate::cli::commands::build::execute(sdk, build_cmd);
    if test_build.is_err() && !cmd.build_first {
        output::warning("Build failed, but continuing with test execution");
    } else if let Err(e) = test_build {
        return Err(e);
    }
    
    // 2. Discover test files
    let test_files = sdk.discover_tests()?;
    output::info(&format!("Found {} test files", test_files.len()));
    
    if test_files.is_empty() {
        output::warning("No test files found");
        return Ok(());
    }
    
    // 3. Compile test files
    output::info("Compiling tests...");
    let mut test_binaries = Vec::new();
    
    for test_file in &test_files {
        output::debug(&format!("Compiling {}", test_file.display()));
        let binary = sdk.compile_test(test_file, cmd.release)?;
        test_binaries.push(binary);
    }
    
    // 4. Run tests
    output::info("Running tests...");
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut ignored_tests = 0;
    
    let test_start = std::time::Instant::now();
    
    for binary in &test_binaries {
        let mut test_args = vec![];
        
        // Add filter if provided
        if let Some(ref filter) = cmd.filter {
            test_args.push("--filter");
            test_args.push(filter);
        }
        
        // Add ignored flag if needed
        if cmd.ignored {
            test_args.push("--ignored");
        }
        
        // Add test threads configuration
        if let Some(threads) = cmd.test_threads {
            test_args.push("--test-threads");
            test_args.push(&threads.to_string());
        }
        
        // Run test binary
        let result = std::process::Command::new(binary)
            .args(&test_args)
            .current_dir(sdk.workspace_path())
            .output();
        
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // Parse test output
                let test_results = parse_test_output(&stdout, &stderr);
                total_tests += test_results.total;
                passed_tests += test_results.passed;
                failed_tests += test_results.failed;
                ignored_tests += test_results.ignored;
                
                // Print test output if verbose
                if cmd.verbose {
                    println!("{}", stdout);
                    if !stderr.is_empty() {
                        eprintln!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                output::error(&format!("Failed to run test {:?}: {}", binary, e));
                if cmd.fail_fast {
                    return Err(SdkError::ExecutionError(format!("Test execution failed: {}", e)));
                }
            }
        }
    }
    
    let test_time = test_start.elapsed();
    
    // 5. Report results
    output::info(&format!("Test results: {} total, {} passed, {} failed, {} ignored",
        total_tests, passed_tests, failed_tests, ignored_tests));
    output::debug(&format!("Test execution time: {:?}", test_time));
    
    if failed_tests > 0 {
        output::error(&format!("{} test(s) failed", failed_tests));
        
        if !cmd.no_fail {
            return Err(SdkError::TestFailed(format!("{} test(s) failed", failed_tests)));
        }
    } else {
        output::success("All tests passed");
    }
    
    // 6. Generate coverage report if requested
    if cmd.coverage {
        output::info("Generating coverage report...");
        sdk.generate_coverage_report(&test_binaries)?;
        output::success("Coverage report generated");
    }
    
    // 7. Generate test report if requested
    if cmd.report.is_some() {
        output::info("Generating test report...");
        sdk.generate_test_report(&test_binaries, cmd.report.as_deref())?;
        output::success("Test report generated");
    }
    
    Ok(())
}

/// Test result summary
struct TestResults {
    total: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
}

/// Parse test output to extract results
fn parse_test_output(stdout: &str, stderr: &str) -> TestResults {
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;
    
    // Parse stdout for test results
    for line in stdout.lines() {
        if line.contains("test result:") {
            // Parse test result line: "test result: ok. X passed; Y failed; Z ignored"
            if line.contains("ok.") {
                if let Some(passed_str) = line.split("passed").next() {
                    if let Some(num) = passed_str.split_whitespace().last() {
                        if let Ok(n) = num.parse::<usize>() {
                            passed = n;
                        }
                    }
                }
                if let Some(ignored_str) = line.split("ignored").next() {
                    if let Some(num) = ignored_str.split_whitespace().last() {
                        if let Ok(n) = num.parse::<usize>() {
                            ignored = n;
                        }
                    }
                }
            } else if line.contains("FAILED") {
                if let Some(failed_str) = line.split("failed").next() {
                    if let Some(num) = failed_str.split_whitespace().last() {
                        if let Ok(n) = num.parse::<usize>() {
                            failed = n;
                        }
                    }
                }
            }
            total = passed + failed + ignored;
        }
    }
    
    // If no result line found, count individual test run lines
    if total == 0 {
        for line in stdout.lines() {
            if line.starts_with("test ") && (line.contains("... ok") || line.contains("... FAILED")) {
                total += 1;
                if line.contains("... ok") {
                    passed += 1;
                } else {
                    failed += 1;
                }
            } else if line.starts_with("test ") && line.contains("... ignored") {
                ignored += 1;
            }
        }
        total += ignored;
    }
    
    TestResults {
        total,
        passed,
        failed,
        ignored,
    }
}
