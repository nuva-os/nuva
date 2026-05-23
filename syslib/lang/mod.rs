/*
 * Nuva OS - SystemLibrary - Lang
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


// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod binary;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod semantic;
pub mod stdlib;

/// InitializeLanguage Runtime
pub fn init_lang() {
    // InitializeLexical Analyzer
    lexer::init_lexer();
    
    // InitializeSyntax Analyzer
    parser::init_parser();
    
    // InitializeSemantic Analysis
    semantic::init_semantic();
    
    // InitializeCode Generation
    codegen::init_codegen();
    
    // InitializeBinary Module
    binary::init_binary();
    
    // InitializeRuntime
    runtime::init_runtime();
    
    // InitializeStandard Library
    stdlib::init_stdlib();
    
    log_info!("Language runtime initialized");
    log_info!("  Execution mode: Native binary (no VM)");
}