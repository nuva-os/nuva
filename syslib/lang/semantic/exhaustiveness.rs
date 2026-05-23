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

use crate::nuva_lang::parser::ast::{Pattern, Literal};
use crate::nuva_lang::semantic::types::Type;

/// Exhaustiveness Checker
/// Verifies that pattern matching is exhaustive, i.e., all possible
/// values are covered by the patterns. This prevents runtime errors
/// from unhandled cases.

/// Pattern Space
/// Represents a set of values that a pattern can match.
/// Used for exhaustiveness checking.
#[derive(Debug, Clone)]
pub enum PatternSpace {
    /// All values of a type
    Universe(Type),
    /// No values (empty set)
    Empty,
    /// Single literal value
    Literal(Literal),
    /// Range of values
    Range { start: Literal, end: Literal, inclusive: bool },
    /// Constructor pattern space
    Constructor {
        name: &'static str,
        args: Vec<PatternSpace>,
    },
    /// Tuple pattern space
    Tuple(Vec<PatternSpace>),
    /// Union of pattern spaces
    Union(Vec<PatternSpace>),
    /// Intersection of pattern spaces
    Intersection(Vec<PatternSpace>),
}

impl PatternSpace {
    /// Create pattern space from a type
    pub fn from_type(ty: &Type) -> Self {
        PatternSpace::Universe(ty.clone())
    }

    /// Create pattern space from a pattern
    pub fn from_pattern(pattern: &Pattern) -> Self {
        match pattern {
            Pattern::Wildcard => PatternSpace::Universe(Type::unknown()),
            Pattern::Literal(lit) => PatternSpace::Literal(lit.clone()),
            Pattern::Identifier(_) => PatternSpace::Universe(Type::unknown()),
            Pattern::Variant { name, fields } => {
                PatternSpace::Constructor {
                    name,
                    args: fields.iter().map(PatternSpace::from_pattern).collect(),
                }
            }
            Pattern::Struct { name, fields } => {
                PatternSpace::Constructor {
                    name,
                    args: fields.iter().map(|(_, p)| PatternSpace::from_pattern(p)).collect(),
                }
            }
            Pattern::Range { start, end, inclusive } => {
                PatternSpace::Range {
                    start: start.clone(),
                    end: end.clone(),
                    inclusive: *inclusive,
                }
            }
            Pattern::Tuple(elements) => {
                PatternSpace::Tuple(elements.iter().map(PatternSpace::from_pattern).collect())
            }
            Pattern::Or(patterns) => {
                PatternSpace::Union(patterns.iter().map(PatternSpace::from_pattern).collect())
            }
        }
    }

    /// Remove a pattern from this space
    pub fn remove_pattern(&self, pattern: &Pattern) -> Vec<PatternSpace> {
        let pattern_space = PatternSpace::from_pattern(pattern);
        self.subtract(&pattern_space)
    }

    /// Subtract another pattern space from this one
    pub fn subtract(&self, other: &PatternSpace) -> Vec<PatternSpace> {
        match (self, other) {
            // Empty - anything = Empty
            (PatternSpace::Empty, _) => vec![PatternSpace::Empty],

            // Anything - Universe = Empty
            (_, PatternSpace::Universe(_)) => vec![PatternSpace::Empty],

            // Universe - Empty = Universe
            (PatternSpace::Universe(ty), PatternSpace::Empty) => {
                vec![PatternSpace::Universe(ty.clone())]
            }

            // Universe - Literal = Universe (for now, simplified)
            (PatternSpace::Universe(ty), PatternSpace::Literal(_)) => {
                // In a complete implementation, this would return Universe minus the literal
                vec![PatternSpace::Universe(ty.clone())]
            }

            // Literal - Literal
            (PatternSpace::Literal(a), PatternSpace::Literal(b)) => {
                if a == b {
                    vec![PatternSpace::Empty]
                } else {
                    vec![PatternSpace::Literal(a.clone())]
                }
            }

            // Constructor - Constructor
            (
                PatternSpace::Constructor { name: n1, args: a1 },
                PatternSpace::Constructor { name: n2, args: a2 },
            ) => {
                if n1 == n2 && a1.len() == a2.len() {
                    // Subtract each argument
                    let mut result = Vec::new();
                    for (s1, s2) in a1.iter().zip(a2.iter()) {
                        result.extend(s1.subtract(s2));
                    }
                    result
                } else {
                    vec![PatternSpace::Constructor {
                        name: n1,
                        args: a1.clone(),
                    }]
                }
            }

            // Tuple - Tuple
            (PatternSpace::Tuple(e1), PatternSpace::Tuple(e2)) => {
                if e1.len() == e2.len() {
                    let mut result = Vec::new();
                    for (s1, s2) in e1.iter().zip(e2.iter()) {
                        result.extend(s1.subtract(s2));
                    }
                    result
                } else {
                    vec![PatternSpace::Tuple(e1.clone())]
                }
            }

            // Union subtraction
            (self_space, PatternSpace::Union(spaces)) => {
                let mut current = vec![self_space.clone()];
                for space in spaces {
                    let mut next = Vec::new();
                    for s in &current {
                        next.extend(s.subtract(space));
                    }
                    current = next;
                }
                current
            }

            // Default: return self unchanged
            _ => vec![self.clone()],
        }
    }

    /// Check if this space is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, PatternSpace::Empty)
    }
}

/// Exhaustiveness Error
#[derive(Debug, Clone)]
pub enum ExhaustivenessError {
    /// Non-exhaustive patterns
    NonExhaustive(Vec<PatternSpace>),
    /// Unreachable pattern
    Unreachable(usize),
    /// Overlapping patterns
    Overlapping(usize, usize),
}

/// Check exhaustiveness of patterns
/// Returns Ok(()) if patterns are exhaustive, Err otherwise.
pub fn check_exhaustiveness(
    ty: &Type,
    patterns: &[Pattern],
) -> Result<(), ExhaustivenessError> {
    // Start with the entire type space
    let mut uncovered = vec![PatternSpace::from_type(ty)];

    // Remove each pattern from the uncovered space
    for pattern in patterns {
        let mut new_uncovered = Vec::new();
        for space in &uncovered {
            new_uncovered.extend(space.remove_pattern(pattern));
        }
        uncovered = new_uncovered;
    }

    // Check if any values remain uncovered
    if uncovered.iter().all(|s| s.is_empty()) {
        Ok(())
    } else {
        Err(ExhaustivenessError::NonExhaustive(uncovered))
    }
}

/// Check for unreachable patterns
/// Returns a list of indices of unreachable patterns.
pub fn check_unreachable(patterns: &[Pattern]) -> Vec<usize> {
    let mut unreachable = Vec::new();
    let mut covered = PatternSpace::Empty;

    for (i, pattern) in patterns.iter().enumerate() {
        let pattern_space = PatternSpace::from_pattern(pattern);

        // Check if this pattern is already covered
        let remaining = pattern_space.subtract(&covered);
        if remaining.iter().all(|s| s.is_empty()) {
            unreachable.push(i);
        } else {
            // Add this pattern to the covered space
            covered = PatternSpace::Union(vec![covered, pattern_space]);
        }
    }

    unreachable
}

/// Check for overlapping patterns
/// Returns a list of pairs of indices of overlapping patterns.
pub fn check_overlapping(patterns: &[Pattern]) -> Vec<(usize, usize)> {
    let mut overlapping = Vec::new();

    for i in 0..patterns.len() {
        for j in (i + 1)..patterns.len() {
            let space_i = PatternSpace::from_pattern(&patterns[i]);
            let space_j = PatternSpace::from_pattern(&patterns[j]);

            // Check if the intersection is non-empty
            let intersection = PatternSpace::Intersection(vec![space_i, space_j]);
            if !intersection.is_empty() {
                overlapping.push((i, j));
            }
        }
    }

    overlapping
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nuva_lang::semantic::types::TypeKind;

    #[test]
    fn test_pattern_space_from_wildcard() {
        let pattern = Pattern::Wildcard;
        let space = PatternSpace::from_pattern(&pattern);
        assert!(matches!(space, PatternSpace::Universe(_)));
    }

    #[test]
    fn test_pattern_space_from_literal() {
        let pattern = Pattern::Literal(Literal::Integer(42));
        let space = PatternSpace::from_pattern(&pattern);
        assert!(matches!(space, PatternSpace::Literal(_)));
    }

    #[test]
    fn test_pattern_space_subtract() {
        let universe = PatternSpace::Universe(Type::int());
        let literal = PatternSpace::Literal(Literal::Integer(42));

        let result = universe.subtract(&literal);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_exhaustiveness_wildcard() {
        let ty = Type::int();
        let patterns = vec![Pattern::Wildcard];

        let result = check_exhaustiveness(&ty, &patterns);
        assert!(result.is_ok());
    }

    #[test]
    fn test_exhaustiveness_non_exhaustive() {
        let ty = Type::int();
        let patterns = vec![Pattern::Literal(Literal::Integer(42))];

        let result = check_exhaustiveness(&ty, &patterns);
        assert!(result.is_err());
    }
}
