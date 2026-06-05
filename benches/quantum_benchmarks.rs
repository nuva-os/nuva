/*
 * Nuva OS - Benches - QuantumBenchmarks
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
 * Quantum Algorithm Performance Benchmarks
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Performance benchmarks for CRYSTALS-Kyber and CRYSTALS-Dilithium
 */

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hal::quantum::pqc::*;

// Kyber benchmarks
fn kyber_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("kyber");

    // Kyber-512
    let kyber512 = Kyber::new(KyberVariant::Kyber512);
    group.bench_with_input(
        BenchmarkId::new("keygen", "Kyber512"),
        &kyber512,
        |b, kyber| {
            b.iter(|| kyber.keygen());
        },
    );

    let (pk512, sk512) = kyber512.keygen().unwrap();
    group.bench_with_input(
        BenchmarkId::new("encapsulate", "Kyber512"),
        &kyber512,
        |b, kyber| {
            b.iter(|| kyber.encapsulate(black_box(&pk512)));
        },
    );

    let (ss512, ct512) = kyber512.encapsulate(&pk512).unwrap();
    group.bench_with_input(
        BenchmarkId::new("decapsulate", "Kyber512"),
        &kyber512,
        |b, kyber| {
            b.iter(|| kyber.decapsulate(black_box(&sk512), black_box(&ct512)));
        },
    );

    // Kyber-768
    let kyber768 = Kyber::new(KyberVariant::Kyber768);
    group.bench_with_input(
        BenchmarkId::new("keygen", "Kyber768"),
        &kyber768,
        |b, kyber| {
            b.iter(|| kyber.keygen());
        },
    );

    let (pk768, sk768) = kyber768.keygen().unwrap();
    group.bench_with_input(
        BenchmarkId::new("encapsulate", "Kyber768"),
        &kyber768,
        |b, kyber| {
            b.iter(|| kyber.encapsulate(black_box(&pk768)));
        },
    );

    let (ss768, ct768) = kyber768.encapsulate(&pk768).unwrap();
    group.bench_with_input(
        BenchmarkId::new("decapsulate", "Kyber768"),
        &kyber768,
        |b, kyber| {
            b.iter(|| kyber.decapsulate(black_box(&sk768), black_box(&ct768)));
        },
    );

    // Kyber-1024
    let kyber1024 = Kyber::new(KyberVariant::Kyber1024);
    group.bench_with_input(
        BenchmarkId::new("keygen", "Kyber1024"),
        &kyber1024,
        |b, kyber| {
            b.iter(|| kyber.keygen());
        },
    );

    let (pk1024, sk1024) = kyber1024.keygen().unwrap();
    group.bench_with_input(
        BenchmarkId::new("encapsulate", "Kyber1024"),
        &kyber1024,
        |b, kyber| {
            b.iter(|| kyber.encapsulate(black_box(&pk1024)));
        },
    );

    let (ss1024, ct1024) = kyber1024.encapsulate(&pk1024).unwrap();
    group.bench_with_input(
        BenchmarkId::new("decapsulate", "Kyber1024"),
        &kyber1024,
        |b, kyber| {
            b.iter(|| kyber.decapsulate(black_box(&sk1024), black_box(&ct1024)));
        },
    );

    group.finish();
}

// Dilithium benchmarks
fn dilithium_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("dilithium");

    let message = b"Benchmark message for Dilithium signature";

    // Dilithium-2
    let dilithium2 = Dilithium::new(DilithiumVariant::Dilithium2);
    group.bench_with_input(
        BenchmarkId::new("keygen", "Dilithium2"),
        &dilithium2,
        |b, dilithium| {
            b.iter(|| dilithium.keygen());
        },
    );

    let (pk2, sk2) = dilithium2.keygen().unwrap();
    group.bench_with_input(
        BenchmarkId::new("sign", "Dilithium2"),
        &dilithium2,
        |b, dilithium| {
            b.iter(|| dilithium.sign(black_box(&sk2), black_box(message)));
        },
    );

    let sig2 = dilithium2.sign(&sk2, message).unwrap();
    group.bench_with_input(
        BenchmarkId::new("verify", "Dilithium2"),
        &dilithium2,
        |b, dilithium| {
            b.iter(|| dilithium.verify(black_box(&pk2), black_box(message), black_box(&sig2)));
        },
    );

    // Dilithium-3
    let dilithium3 = Dilithium::new(DilithiumVariant::Dilithium3);
    group.bench_with_input(
        BenchmarkId::new("keygen", "Dilithium3"),
        &dilithium3,
        |b, dilithium| {
            b.iter(|| dilithium.keygen());
        },
    );

    let (pk3, sk3) = dilithium3.keygen().unwrap();
    group.bench_with_input(
        BenchmarkId::new("sign", "Dilithium3"),
        &dilithium3,
        |b, dilithium| {
            b.iter(|| dilithium.sign(black_box(&sk3), black_box(message)));
        },
    );

    let sig3 = dilithium3.sign(&sk3, message).unwrap();
    group.bench_with_input(
        BenchmarkId::new("verify", "Dilithium3"),
        &dilithium3,
        |b, dilithium| {
            b.iter(|| dilithium.verify(black_box(&pk3), black_box(message), black_box(&sig3)));
        },
    );

    // Dilithium-5
    let dilithium5 = Dilithium::new(DilithiumVariant::Dilithium5);
    group.bench_with_input(
        BenchmarkId::new("keygen", "Dilithium5"),
        &dilithium5,
        |b, dilithium| {
            b.iter(|| dilithium.keygen());
        },
    );

    let (pk5, sk5) = dilithium5.keygen().unwrap();
    group.bench_with_input(
        BenchmarkId::new("sign", "Dilithium5"),
        &dilithium5,
        |b, dilithium| {
            b.iter(|| dilithium.sign(black_box(&sk5), black_box(message)));
        },
    );

    let sig5 = dilithium5.sign(&sk5, message).unwrap();
    group.bench_with_input(
        BenchmarkId::new("verify", "Dilithium5"),
        &dilithium5,
        |b, dilithium| {
            b.iter(|| dilithium.verify(black_box(&pk5), black_box(message), black_box(&sig5)));
        },
    );

    group.finish();
}

// Throughput benchmarks
fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Kyber-768 operations per second
    let kyber = Kyber::new(KyberVariant::Kyber768);
    let (pk, sk) = kyber.keygen().unwrap();
    let (ss, ct) = kyber.encapsulate(&pk).unwrap();

    group.bench_function("kyber768_keygen_ops", |b| {
        b.iter(|| kyber.keygen());
    });

    group.bench_function("kyber768_encaps_ops", |b| {
        b.iter(|| kyber.encapsulate(&pk));
    });

    group.bench_function("kyber768_decaps_ops", |b| {
        b.iter(|| kyber.decapsulate(&sk, &ct));
    });

    // Dilithium-3 operations per second
    let dilithium = Dilithium::new(DilithiumVariant::Dilithium3);
    let (pk, sk) = dilithium.keygen().unwrap();
    let msg = b"Test message";
    let sig = dilithium.sign(&sk, msg).unwrap();

    group.bench_function("dilithium3_keygen_ops", |b| {
        b.iter(|| dilithium.keygen());
    });

    group.bench_function("dilithium3_sign_ops", |b| {
        b.iter(|| dilithium.sign(&sk, msg));
    });

    group.bench_function("dilithium3_verify_ops", |b| {
        b.iter(|| dilithium.verify(&pk, msg, &sig));
    });

    group.finish();
}

// Memory benchmarks
fn memory_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // Key sizes
    group.bench_function("kyber768_public_key_size", |b| {
        b.iter(|| black_box(KyberVariant::Kyber768.public_key_size()));
    });

    group.bench_function("kyber768_secret_key_size", |b| {
        b.iter(|| black_box(KyberVariant::Kyber768.secret_key_size()));
    });

    group.bench_function("kyber768_ciphertext_size", |b| {
        b.iter(|| black_box(KyberVariant::Kyber768.ciphertext_size()));
    });

    group.bench_function("dilithium3_public_key_size", |b| {
        b.iter(|| black_box(DilithiumVariant::Dilithium3.public_key_size()));
    });

    group.bench_function("dilithium3_secret_key_size", |b| {
        b.iter(|| black_box(DilithiumVariant::Dilithium3.secret_key_size()));
    });

    group.bench_function("dilithium3_signature_size", |b| {
        b.iter(|| black_box(DilithiumVariant::Dilithium3.signature_size()));
    });

    group.finish();
}

// Comparison benchmarks
fn comparison_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");

    // Compare all Kyber variants
    for variant in [
        KyberVariant::Kyber512,
        KyberVariant::Kyber768,
        KyberVariant::Kyber1024,
    ] {
        let kyber = Kyber::new(variant);
        let name = format!("{:?}", variant);

        group.bench_with_input(
            BenchmarkId::new("kyber_keygen", &name),
            &kyber,
            |b, kyber| {
                b.iter(|| kyber.keygen());
            },
        );
    }

    // Compare all Dilithium variants
    for variant in [
        DilithiumVariant::Dilithium2,
        DilithiumVariant::Dilithium3,
        DilithiumVariant::Dilithium5,
    ] {
        let dilithium = Dilithium::new(variant);
        let name = format!("{:?}", variant);

        group.bench_with_input(
            BenchmarkId::new("dilithium_keygen", &name),
            &dilithium,
            |b, dilithium| {
                b.iter(|| dilithium.keygen());
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    kyber_benchmarks,
    dilithium_benchmarks,
    throughput_benchmarks,
    memory_benchmarks,
    comparison_benchmarks,
);

criterion_main!(benches);
