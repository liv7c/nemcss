# Benchmark Report: cli

**Date:** 2026-03-07 12:44:36 CET
**Git Branch:** `feature/make_adding_utilities_explicit_with_semantic_layer`
**Git Commit:** `8e855ff (dirty)`
**Description:** after_create_utility_refactoring

## Environment

| Property | Value |
|----------|-------|
| Platform | Darwin |
| Machine | arm64 |
| OS Version | 25.3.0 |
| Rust Version | rustc 1.93.0 (254b59607 2026-01-19) |

## Benchmark Results

| Benchmark | Fastest | Median | Mean | Slowest |
|-----------|---------|--------|------|---------|
| extra_large_project | 57.26 ms | 58.65 ms | 59.86 ms | 81.84 ms |
| large_project | 25.63 ms | 26.46 ms | 26.56 ms | 31.28 ms |
| large_project_with_semantic | 25.63 ms | 26.33 ms | 26.35 ms | 27.43 ms |
| medium_project | 6.888 ms | 7.056 ms | 7.319 ms | 32.4 ms |
| medium_project_with_semantic | 6.903 ms | 7.047 ms | 7.067 ms | 8.358 ms |
| small_project | 1.306 ms | 1.381 ms | 1.403 ms | 1.704 ms |
| small_project_with_semantic | 1.307 ms | 1.381 ms | 1.417 ms | 2.846 ms |

<details>
<summary>Full Benchmark Details (click to expand)</summary>

```
Timer precision: 41 ns
build_command                    fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ extra_large_project           57.26 ms      │ 81.84 ms      │ 58.65 ms      │ 59.86 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  16627       │ 16627         │ 16627         │ 16629         │         │
│                                  1.33 MB     │ 1.33 MB       │ 1.33 MB       │ 1.331 MB      │         │
│                                alloc:        │               │               │               │         │
│                                  34264       │ 34264         │ 34264         │ 34281         │         │
│                                  1.589 MB    │ 1.589 MB      │ 1.589 MB      │ 1.591 MB      │         │
│                                dealloc:      │               │               │               │         │
│                                  41270       │ 41270         │ 41270         │ 41285         │         │
│                                  3.74 MB     │ 3.74 MB       │ 3.74 MB       │ 3.742 MB      │         │
│                                grow:         │               │               │               │         │
│                                  10698       │ 10698         │ 10698         │ 10699         │         │
│                                  1.638 MB    │ 1.638 MB      │ 1.638 MB      │ 1.639 MB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15.1          │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 2.023 KB      │         │
├─ large_project                 25.63 ms      │ 31.28 ms      │ 26.46 ms      │ 26.56 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  14828       │ 14828         │ 14828         │ 14828         │         │
│                                  1.177 MB    │ 1.177 MB      │ 1.177 MB      │ 1.177 MB      │         │
│                                alloc:        │               │               │               │         │
│                                  30114       │ 30114         │ 30114         │ 30114         │         │
│                                  1.389 MB    │ 1.389 MB      │ 1.389 MB      │ 1.389 MB      │         │
│                                dealloc:      │               │               │               │         │
│                                  37119       │ 37119         │ 37119         │ 37119         │         │
│                                  3.342 MB    │ 3.342 MB      │ 3.342 MB      │ 3.342 MB      │         │
│                                grow:         │               │               │               │         │
│                                  7939        │ 7939          │ 7939          │ 7939          │         │
│                                  1.441 MB    │ 1.441 MB      │ 1.441 MB      │ 1.441 MB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ large_project_with_semantic   25.63 ms      │ 27.43 ms      │ 26.33 ms      │ 26.35 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  14870       │ 14870         │ 14870         │ 14870         │         │
│                                  1.199 MB    │ 1.199 MB      │ 1.199 MB      │ 1.199 MB      │         │
│                                alloc:        │               │               │               │         │
│                                  30167       │ 30167         │ 30167         │ 30167         │         │
│                                  1.393 MB    │ 1.393 MB      │ 1.393 MB      │ 1.393 MB      │         │
│                                dealloc:      │               │               │               │         │
│                                  37172       │ 37172         │ 37172         │ 37172         │         │
│                                  3.366 MB    │ 3.366 MB      │ 3.366 MB      │ 3.366 MB      │         │
│                                grow:         │               │               │               │         │
│                                  7961        │ 7961          │ 7961          │ 7961          │         │
│                                  1.461 MB    │ 1.461 MB      │ 1.461 MB      │ 1.461 MB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ medium_project                6.888 ms      │ 32.4 ms       │ 7.056 ms      │ 7.319 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  7157        │ 7157          │ 7157          │ 7157          │         │
│                                  516.3 KB    │ 516.3 KB      │ 516.3 KB      │ 516.3 KB      │         │
│                                alloc:        │               │               │               │         │
│                                  15409       │ 15409         │ 15409         │ 15409         │         │
│                                  915.9 KB    │ 915.9 KB      │ 915.9 KB      │ 916 KB        │         │
│                                dealloc:      │               │               │               │         │
│                                  20731       │ 20731         │ 20731         │ 20731         │         │
│                                  1.662 MB    │ 1.662 MB      │ 1.662 MB      │ 1.662 MB      │         │
│                                grow:         │               │               │               │         │
│                                  2945        │ 2945          │ 2945          │ 2945          │         │
│                                  463.9 KB    │ 463.9 KB      │ 463.9 KB      │ 463.9 KB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ medium_project_with_semantic  6.903 ms      │ 8.358 ms      │ 7.047 ms      │ 7.067 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  7199        │ 7199          │ 7199          │ 7199          │         │
│                                  524.6 KB    │ 524.6 KB      │ 524.6 KB      │ 524.6 KB      │         │
│                                alloc:        │               │               │               │         │
│                                  15462       │ 15462         │ 15462         │ 15462         │         │
│                                  920.1 KB    │ 920.1 KB      │ 920.1 KB      │ 920.1 KB      │         │
│                                dealloc:      │               │               │               │         │
│                                  20784       │ 20784         │ 20784         │ 20784         │         │
│                                  1.673 MB    │ 1.673 MB      │ 1.673 MB      │ 1.673 MB      │         │
│                                grow:         │               │               │               │         │
│                                  2967        │ 2967          │ 2967          │ 2967          │         │
│                                  470.9 KB    │ 470.9 KB      │ 470.9 KB      │ 470.9 KB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ small_project                 1.306 ms      │ 1.704 ms      │ 1.381 ms      │ 1.403 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  579         │ 579           │ 579           │ 579           │         │
│                                  62.18 KB    │ 62.18 KB      │ 62.18 KB      │ 62.18 KB      │         │
│                                alloc:        │               │               │               │         │
│                                  2458        │ 2458          │ 2458          │ 2458          │         │
│                                  185.6 KB    │ 185.6 KB      │ 185.6 KB      │ 185.7 KB      │         │
│                                dealloc:      │               │               │               │         │
│                                  2890        │ 2890          │ 2890          │ 2890          │         │
│                                  258.3 KB    │ 258.3 KB      │ 258.3 KB      │ 258.3 KB      │         │
│                                grow:         │               │               │               │         │
│                                  727         │ 727           │ 727           │ 727           │         │
│                                  56.18 KB    │ 56.18 KB      │ 56.18 KB      │ 56.18 KB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
╰─ small_project_with_semantic   1.307 ms      │ 2.846 ms      │ 1.381 ms      │ 1.417 ms      │ 100     │ 100
                                 max alloc:    │               │               │               │         │
                                   596         │ 596           │ 596           │ 596           │         │
                                   63.14 KB    │ 63.14 KB      │ 63.14 KB      │ 63.14 KB      │         │
                                 alloc:        │               │               │               │         │
                                   2511        │ 2511          │ 2511          │ 2511          │         │
                                   189.8 KB    │ 189.8 KB      │ 189.8 KB      │ 189.8 KB      │         │
                                 dealloc:      │               │               │               │         │
                                   2943        │ 2943          │ 2943          │ 2943          │         │
                                   264.2 KB    │ 264.2 KB      │ 264.2 KB      │ 264.2 KB      │         │
                                 grow:         │               │               │               │         │
                                   749         │ 749           │ 749           │ 749           │         │
                                   57.9 KB     │ 57.9 KB       │ 57.9 KB       │ 57.9 KB       │         │
                                 shrink:       │               │               │               │         │
                                   15          │ 15            │ 15            │ 15            │         │
                                   1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
```

</details>

## Summary

| Benchmark | Median Time |
|-----------|-------------|
| extra_large_project | 58.65 ms |
| large_project | 26.46 ms |
| large_project_with_semantic | 26.33 ms |
| medium_project | 7.056 ms |
| medium_project_with_semantic | 7.047 ms |
| small_project | 1.381 ms |
| small_project_with_semantic | 1.381 ms |
