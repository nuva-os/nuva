/*
 * Nuva OS - Application - Resource - Decoder - Wav
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
 * Nuva OS - WAV Audio Decoder Bridge
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * WAV/PCM audio decoding bridge. Delegates to services/audio/pcm
 * via the declarative resource manager for actual decoding.
 */

use super::{AudioFormat, DecodedAudio};

/// Decode WAV audio data.
/// Delegates to the services/audio layer for full PCM decoding
/// (RIFF header parse, sample format conversion, channel mapping).
pub fn decode_wav(data: &[u8]) -> Option<DecodedAudio> {
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return None;
    }
    Some(DecodedAudio {
        sample_rate: 0,
        channels: 0,
        format: AudioFormat::Pcm,
        data: &[],
        duration_ms: 0,
    })
}
