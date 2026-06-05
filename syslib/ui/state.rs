/*
 * Nuva OS - Syslib - Ui - State
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
/*
 * Nuva OS - Syslib - UI - State
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Reactive State<T> for declarative UI binding.
 */

use core::sync::atomic::{AtomicU32, Ordering};

/** Reactive State<T> — atomic version + dirty marking.
 *
 * When set() changes the value, the version increments and
 * the dirty flag is set, triggering a re-render.
 *
 * Constraint: T: Copy + PartialEq (no heap types in no_std).
 */
pub struct State<T: Copy + PartialEq> {
    /** Current value. */
    value: T,
    /** Version counter. */
    version: AtomicU32,
    /** Dirty flag. */
    dirty: AtomicU32,
}

impl<T: Copy + PartialEq> State<T> {
    /** Create a new State with initial value. */
    pub const fn new(value: T) -> Self {
        State {
            value,
            version: AtomicU32::new(0),
            dirty: AtomicU32::new(0),
        }
    }

    /** Get the current value. */
    pub fn get(&self) -> T { self.value }

    /** Set a new value. Marks dirty only if value changes. */
    pub fn set(&mut self, value: T) {
        if self.value != value {
            self.value = value;
            self.version.fetch_add(1, Ordering::AcqRel);
            self.dirty.store(1, Ordering::Release);
        }
    }

    /** Get the version number. */
    pub fn version(&self) -> u32 { self.version.load(Ordering::Acquire) }

    /** Check and consume the dirty flag. */
    pub fn take_dirty(&self) -> bool {
        self.dirty.swap(0, Ordering::AcqRel) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_initial() {
        let state: State<u32> = State::new(42);
        assert_eq!(state.get(), 42);
        assert_eq!(state.version(), 0);
        assert!(!state.take_dirty());
    }

    #[test]
    fn test_state_set_change() {
        let mut state: State<u32> = State::new(0);
        state.set(10);
        assert_eq!(state.get(), 10);
        assert_eq!(state.version(), 1);
        assert!(state.take_dirty());
        assert!(!state.take_dirty());
    }

    #[test]
    fn test_state_set_same() {
        let mut state: State<u32> = State::new(5);
        state.set(5);
        assert_eq!(state.version(), 0);
        assert!(!state.take_dirty());
    }
}
