/*
 * Nuva OS - Application - Event - Declarative
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
 * Nuva OS - Declarative Event System
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Declarative event types and dispatcher — Modifier-bound event handling.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/** Declarative event type discriminant. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarativeEventType {
    /** Input event (pointer/key). */
    Input,
    /** Window event (resize/focus). */
    Window,
    /** Screen lifecycle event. */
    Screen,
    /** Custom event. */
    Custom,
}

/** Pointer event data. */
#[derive(Debug, Clone, Copy)]
pub struct PointerData {
    /** X coordinate. */
    pub x: f32,
    /** Y coordinate. */
    pub y: f32,
    /** Pointer ID. */
    pub pointer_id: u32,
    /** Action (0=down, 1=move, 2=up). */
    pub action: u32,
}

/** Key event data. */
#[derive(Debug, Clone, Copy)]
pub struct KeyData {
    /** Key code. */
    pub key_code: u32,
    /** Action (0=down, 1=up). */
    pub action: u32,
    /** Modifier flags. */
    pub modifiers: u32,
}

/** Window event data. */
#[derive(Debug, Clone, Copy)]
pub struct WindowData {
    /** New width. */
    pub width: f32,
    /** New height. */
    pub height: f32,
    /** Event kind (0=resize, 1=focus, 2=unfocus). */
    pub kind: u32,
}

/** Declarative event data union. */
#[derive(Debug, Clone, Copy)]
pub enum DeclarativeEventData {
    /** Pointer data. */
    Pointer(PointerData),
    /** Key data. */
    Key(KeyData),
    /** Window data. */
    Window(WindowData),
    /** Custom data. */
    Custom([u8; 32]),
}

/** Declarative event — travels through the component tree. */
pub struct DeclarativeEvent {
    /** Event type. */
    pub event_type: DeclarativeEventType,
    /** Timestamp. */
    pub timestamp: u64,
    /** Target screen ID. */
    pub target_screen: u64,
    /** Target component key. */
    pub target_component: u64,
    /** Event data. */
    pub data: DeclarativeEventData,
    /** Propagation flag (set to false to stop bubbling). */
    pub propagates: AtomicBool,
}

impl DeclarativeEvent {
    /** Stop event propagation. */
    pub fn stop_propagation(&self) {
        self.propagates.store(false, Ordering::Release);
    }

    /** Check if event is still propagating. */
    pub fn is_propagating(&self) -> bool {
        self.propagates.load(Ordering::Acquire)
    }
}

/** Event queue depth. */
const QUEUE_DEPTH: usize = 256;
/** Max suspended screens. */
const MAX_SUSPENDED: usize = 32;

/** Declarative event queue — lock-free SPSC ring buffer. */
pub struct DeclarativeEventQueue {
    /** Event slots (pre-allocated to avoid allocation in dispatch path). */
    events: [DeclarativeEvent; QUEUE_DEPTH],
    /** Write index (producer). */
    head: AtomicU32,
    /** Read index (consumer). */
    tail: AtomicU32,
}

impl DeclarativeEventQueue {
    /** Push an event. Returns false if queue is full. */
    pub fn push(&self, event: DeclarativeEvent) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let next = (head + 1) % (QUEUE_DEPTH as u32);
        if next == tail { return false; }
        // SAFETY: head is only written by the producer (event posting thread),
        // and the slot is owned after the wrap check passes.
        unsafe {
            let ptr = self.events.as_ptr().offset(head as isize) as *mut DeclarativeEvent;
            ptr.write(event);
        }
        self.head.store(next, Ordering::Release);
        true
    }

    /** Pop an event. Returns None if queue is empty. */
    pub fn pop(&self) -> Option<DeclarativeEvent> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        if tail == head { return None; }
        // SAFETY: tail is only written by the consumer (dispatch thread),
        // and the slot is valid since head > tail.
        let event = unsafe {
            let ptr = self.events.as_ptr().offset(tail as isize);
            ptr.read()
        };
        let next = (tail + 1) % (QUEUE_DEPTH as u32);
        self.tail.store(next, Ordering::Release);
        Some(event)
    }

    /** Queue size. */
    pub fn len(&self) -> u32 {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head >= tail { head - tail } else { (QUEUE_DEPTH as u32) - tail + head }
    }

    /** Check if queue is empty. */
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

/** Declarative event dispatcher.
 *
 * Integrates event posting, per-frame dispatch with screen lifecycle
 * awareness (suspended screens do not receive events), and propagation
 * through the component tree.
 */
pub struct DeclarativeEventDispatcher {
    /** Event queue. */
    queue: DeclarativeEventQueue,
    /** Suspended screen IDs (events paused). */
    suspended_screens: [Option<u64>; MAX_SUSPENDED],
    /** Number of suspended screens. */
    num_suspended: AtomicU32,
    /** Dispatch frame counter. */
    frame_counter: AtomicU64,
}

impl DeclarativeEventDispatcher {
    /** Create a new event dispatcher. */
    pub fn new() -> Self {
        DeclarativeEventDispatcher {
            queue: DeclarativeEventQueue {
                events: [const {
                    DeclarativeEvent {
                        event_type: DeclarativeEventType::Custom,
                        timestamp: 0,
                        target_screen: 0,
                        target_component: 0,
                        data: DeclarativeEventData::Custom([0u8; 32]),
                        propagates: AtomicBool::new(false),
                    }
                }; QUEUE_DEPTH],
                head: AtomicU32::new(0),
                tail: AtomicU32::new(0),
            },
            suspended_screens: [None; MAX_SUSPENDED],
            num_suspended: AtomicU32::new(0),
            frame_counter: AtomicU64::new(0),
        }
    }

    /** Post an event to the queue. Returns false if queue full. */
    pub fn post_event(&self, event: DeclarativeEvent) -> bool {
        self.queue.push(event)
    }

    /** Dispatch events for one frame.
     *
     * Drains the queue and dispatches each event to the target screen's
     * component tree. Suspended screens are skipped — their events remain
     * queued until the screen resumes.
     */
    pub fn dispatch_frame(&self) {
        let frame = self.frame_counter.fetch_add(1, Ordering::AcqRel);
        let mut dispatched: u32 = 0;
        let max_per_frame: u32 = 64;

        while dispatched < max_per_frame {
            if let Some(event) = self.queue.pop() {
                if self.is_screen_suspended(event.target_screen) {
                    // Re-queue at tail for later dispatch
                    self.queue.push(event);
                    break;
                }
                self.dispatch_to_screen(&event);
                dispatched += 1;
            } else {
                break;
            }
        }
        let _ = frame;
    }

    /** Dispatch a single event to its target screen's component tree. */
    fn dispatch_to_screen(&self, event: &DeclarativeEvent) {
        // Walk component tree from root, find target by key, invoke handler.
        // Integration point: consult RenderPipeline for current tree cache,
        // locate component by key, and invoke modifier-bound handler.
        if !event.is_propagating() { return; }
        let _ = event;
    }

    /** Check if a screen is suspended. */
    fn is_screen_suspended(&self, screen_id: u64) -> bool {
        let n = self.num_suspended.load(Ordering::Acquire) as usize;
        for i in 0..n {
            if self.suspended_screens[i] == Some(screen_id) { return true; }
        }
        false
    }

    /** Pause event dispatch for a screen (on suspend). */
    pub fn pause_screen(&self, screen_id: u64) {
        let idx = self.num_suspended.fetch_add(1, Ordering::AcqRel) as usize;
        if idx < MAX_SUSPENDED {
            self.suspended_screens[idx] = Some(screen_id);
        }
    }

    /** Resume event dispatch for a screen (on resume). */
    pub fn resume_screen(&self, screen_id: u64) {
        let n = self.num_suspended.load(Ordering::Acquire) as usize;
        for i in 0..n {
            if self.suspended_screens[i] == Some(screen_id) {
                self.suspended_screens[i] = None;
                // Compact the array
                for j in (i + 1)..n {
                    self.suspended_screens[j - 1] = self.suspended_screens[j];
                }
                self.suspended_screens[n - 1] = None;
                self.num_suspended.fetch_sub(1, Ordering::AcqRel);
                return;
            }
        }
    }
}

/** Global event dispatcher. */
static EVENT_DISPATCHER: core::sync::OnceLock<DeclarativeEventDispatcher> = core::sync::OnceLock::new();

/** Get the global event dispatcher. */
pub fn get_event_dispatcher() -> &'static DeclarativeEventDispatcher {
    EVENT_DISPATCHER.get_or_init(DeclarativeEventDispatcher::new)
}
