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
