# Benchmark Report: cli

**Date:** 2026-03-07 12:39:22 CET
**Git Branch:** `feature/make_adding_utilities_explicit_with_semantic_layer`
**Git Commit:** `8e855ff (dirty)`
**Description:** after_adding_semantic_system

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
| extra_large_project | 55.01 ms | 58.84 ms | 59.07 ms | 71.33 ms |
| large_project | 25.8 ms | 26.62 ms | 26.71 ms | 30.42 ms |
| large_project_with_semantic | 25.45 ms | 26.65 ms | 26.89 ms | 46.23 ms |
| medium_project | 6.923 ms | 7.173 ms | 7.255 ms | 8.789 ms |
| medium_project_with_semantic | 7.018 ms | 7.511 ms | 8.481 ms | 25.13 ms |
| small_project | 1.308 ms | 1.434 ms | 1.545 ms | 3.997 ms |
| small_project_with_semantic | 1.337 ms | 1.414 ms | 1.442 ms | 1.707 ms |

<details>
<summary>Full Benchmark Details (click to expand)</summary>

```
Timer precision: 41 ns
build_command                    fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ extra_large_project           55.01 ms      │ 71.33 ms      │ 58.84 ms      │ 59.07 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  16627       │ 16627         │ 16627         │ 16629         │         │
│                                  1.33 MB     │ 1.33 MB       │ 1.33 MB       │ 1.331 MB      │         │
│                                alloc:        │               │               │               │         │
│                                  37064       │ 37064         │ 37064         │ 37081         │         │
│                                  1.671 MB    │ 1.671 MB      │ 1.671 MB      │ 1.672 MB      │         │
│                                dealloc:      │               │               │               │         │
│                                  44070       │ 44070         │ 44070         │ 44085         │         │
│                                  3.833 MB    │ 3.833 MB      │ 3.833 MB      │ 3.834 MB      │         │
│                                grow:         │               │               │               │         │
│                                  11198       │ 11198         │ 11198         │ 11199         │         │
│                                  1.649 MB    │ 1.649 MB      │ 1.649 MB      │ 1.65 MB       │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15.1          │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 2.023 KB      │         │
├─ large_project                 25.8 ms       │ 30.42 ms      │ 26.62 ms      │ 26.71 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  14828       │ 14828         │ 14828         │ 14828         │         │
│                                  1.177 MB    │ 1.177 MB      │ 1.177 MB      │ 1.177 MB      │         │
│                                alloc:        │               │               │               │         │
│                                  32914       │ 32914         │ 32914         │ 32914         │         │
│                                  1.47 MB     │ 1.47 MB       │ 1.47 MB       │ 1.47 MB       │         │
│                                dealloc:      │               │               │               │         │
│                                  39919       │ 39919         │ 39919         │ 39919         │         │
│                                  3.434 MB    │ 3.434 MB      │ 3.434 MB      │ 3.434 MB      │         │
│                                grow:         │               │               │               │         │
│                                  8439        │ 8439          │ 8439          │ 8439          │         │
│                                  1.452 MB    │ 1.452 MB      │ 1.452 MB      │ 1.452 MB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ large_project_with_semantic   25.45 ms      │ 46.23 ms      │ 26.65 ms      │ 26.89 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  14870       │ 14870         │ 14870         │ 14870         │         │
│                                  1.199 MB    │ 1.199 MB      │ 1.199 MB      │ 1.199 MB      │         │
│                                alloc:        │               │               │               │         │
│                                  32967       │ 32967         │ 32967         │ 32967         │         │
│                                  1.474 MB    │ 1.474 MB      │ 1.474 MB      │ 1.474 MB      │         │
│                                dealloc:      │               │               │               │         │
│                                  39972       │ 39972         │ 39972         │ 39972         │         │
│                                  3.459 MB    │ 3.459 MB      │ 3.459 MB      │ 3.459 MB      │         │
│                                grow:         │               │               │               │         │
│                                  8461        │ 8461          │ 8461          │ 8461          │         │
│                                  1.472 MB    │ 1.472 MB      │ 1.472 MB      │ 1.472 MB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ medium_project                6.923 ms      │ 8.789 ms      │ 7.173 ms      │ 7.255 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  7157        │ 7157          │ 7157          │ 7157          │         │
│                                  516.3 KB    │ 516.3 KB      │ 516.3 KB      │ 516.3 KB      │         │
│                                alloc:        │               │               │               │         │
│                                  16409       │ 16409         │ 16409         │ 16409         │         │
│                                  944 KB      │ 944 KB        │ 944 KB        │ 944 KB        │         │
│                                dealloc:      │               │               │               │         │
│                                  21731       │ 21731         │ 21731         │ 21731         │         │
│                                  1.694 MB    │ 1.694 MB      │ 1.694 MB      │ 1.694 MB      │         │
│                                grow:         │               │               │               │         │
│                                  3095        │ 3095          │ 3095          │ 3095          │         │
│                                  467.6 KB    │ 467.6 KB      │ 467.6 KB      │ 467.6 KB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ medium_project_with_semantic  7.018 ms      │ 25.13 ms      │ 7.511 ms      │ 8.481 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  7199        │ 7199          │ 7199          │ 7199          │         │
│                                  524.6 KB    │ 524.6 KB      │ 524.6 KB      │ 524.6 KB      │         │
│                                alloc:        │               │               │               │         │
│                                  16462       │ 16462         │ 16462         │ 16462         │         │
│                                  948.1 KB    │ 948.1 KB      │ 948.1 KB      │ 948.1 KB      │         │
│                                dealloc:      │               │               │               │         │
│                                  21784       │ 21784         │ 21784         │ 21784         │         │
│                                  1.705 MB    │ 1.705 MB      │ 1.705 MB      │ 1.705 MB      │         │
│                                grow:         │               │               │               │         │
│                                  3117        │ 3117          │ 3117          │ 3117          │         │
│                                  474.6 KB    │ 474.6 KB      │ 474.6 KB      │ 474.6 KB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ small_project                 1.308 ms      │ 3.997 ms      │ 1.434 ms      │ 1.545 ms      │ 100     │ 100
│                                max alloc:    │               │               │               │         │
│                                  579         │ 579           │ 579           │ 579           │         │
│                                  62.18 KB    │ 62.18 KB      │ 62.18 KB      │ 62.18 KB      │         │
│                                alloc:        │               │               │               │         │
│                                  2562        │ 2562          │ 2562          │ 2562          │         │
│                                  188.5 KB    │ 188.5 KB      │ 189.3 KB      │ 188.5 KB      │         │
│                                dealloc:      │               │               │               │         │
│                                  2994        │ 2994          │ 2994          │ 2994          │         │
│                                  261.6 KB    │ 261.6 KB      │ 261.6 KB      │ 261.6 KB      │         │
│                                grow:         │               │               │               │         │
│                                  745         │ 745           │ 745           │ 745           │         │
│                                  56.57 KB    │ 56.57 KB      │ 56.57 KB      │ 56.57 KB      │         │
│                                shrink:       │               │               │               │         │
│                                  15          │ 15            │ 15            │ 15            │         │
│                                  1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
╰─ small_project_with_semantic   1.337 ms      │ 1.707 ms      │ 1.414 ms      │ 1.442 ms      │ 100     │ 100
                                 max alloc:    │               │               │               │         │
                                   597         │ 597           │ 597           │ 597           │         │
                                   63.14 KB    │ 63.14 KB      │ 63.14 KB      │ 63.14 KB      │         │
                                 alloc:        │               │               │               │         │
                                   2615        │ 2615          │ 2615          │ 2615          │         │
                                   192.6 KB    │ 192.6 KB      │ 192.6 KB      │ 192.7 KB      │         │
                                 dealloc:      │               │               │               │         │
                                   3047        │ 3047          │ 3047          │ 3047          │         │
                                   267.4 KB    │ 267.4 KB      │ 267.4 KB      │ 267.4 KB      │         │
                                 grow:         │               │               │               │         │
                                   767         │ 767           │ 767           │ 767           │         │
                                   58.28 KB    │ 58.28 KB      │ 58.28 KB      │ 58.28 KB      │         │
                                 shrink:       │               │               │               │         │
                                   15          │ 15            │ 15            │ 15            │         │
                                   1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
```

</details>

## Summary

| Benchmark | Median Time |
|-----------|-------------|
| extra_large_project | 58.84 ms |
| large_project | 26.62 ms |
| large_project_with_semantic | 26.65 ms |
| medium_project | 7.173 ms |
| medium_project_with_semantic | 7.511 ms |
| small_project | 1.434 ms |
| small_project_with_semantic | 1.414 ms |
