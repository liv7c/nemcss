# Benchmark Report: cli

**Date:** 2026-02-05 21:39:51 CET
**Git Branch:** `feat/cli/add_benchmarks_and_concurrency_build_command`
**Git Commit:** `dcf2d16 (dirty)`
**Description:** benchmark_with_rayon

## Environment

| Property | Value |
|----------|-------|
| Platform | Darwin |
| Machine | arm64 |
| OS Version | 25.2.0 |
| Rust Version | rustc 1.93.0 (254b59607 2026-01-19) |

## Benchmark Results

| Benchmark | Fastest | Median | Mean | Slowest |
|-----------|---------|--------|------|---------|
| extra_large_project | 57.64 ms | 60.44 ms | 61.91 ms | 112 ms |
| large_project | 27.21 ms | 28.62 ms | 29.17 ms | 42.04 ms |
| medium_project | 7.262 ms | 7.551 ms | 7.801 ms | 19.89 ms |
| small_project | 1.259 ms | 1.399 ms | 1.433 ms | 1.82 ms |

<details>
<summary>Full Benchmark Details (click to expand)</summary>

```
Timer precision: 41 ns
build_command           fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ extra_large_project  57.64 ms      │ 112 ms        │ 60.44 ms      │ 61.91 ms      │ 100     │ 100
│                       max alloc:    │               │               │               │         │
│                         19624       │ 19624         │ 19624         │ 19626         │         │
│                         1.585 MB    │ 1.585 MB      │ 1.585 MB      │ 1.586 MB      │         │
│                       alloc:        │               │               │               │         │
│                         44469       │ 44469         │ 44469         │ 44486         │         │
│                         1.889 MB    │ 1.889 MB      │ 1.889 MB      │ 1.891 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         51475       │ 51475         │ 51475         │ 51490         │         │
│                         4.336 MB    │ 4.336 MB      │ 4.336 MB      │ 4.338 MB      │         │
│                       grow:         │               │               │               │         │
│                         14650       │ 14650         │ 14650         │ 14651         │         │
│                         1.934 MB    │ 1.934 MB      │ 1.934 MB      │ 1.935 MB      │         │
│                       shrink:       │               │               │               │         │
│                         15          │ 15            │ 15            │ 15.1          │         │
│                         1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 2.023 KB      │         │
├─ large_project        27.21 ms      │ 42.04 ms      │ 28.62 ms      │ 29.17 ms      │ 100     │ 100
│                       max alloc:    │               │               │               │         │
│                         17825       │ 17825         │ 17825         │ 17825         │         │
│                         1.399 MB    │ 1.399 MB      │ 1.399 MB      │ 1.399 MB      │         │
│                       alloc:        │               │               │               │         │
│                         39569       │ 39569         │ 39569         │ 39569         │         │
│                         1.683 MB    │ 1.683 MB      │ 1.683 MB      │ 1.683 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         46574       │ 46574         │ 46574         │ 46574         │         │
│                         3.926 MB    │ 3.926 MB      │ 3.926 MB      │ 3.926 MB      │         │
│                       grow:         │               │               │               │         │
│                         11141       │ 11141         │ 11141         │ 11141         │         │
│                         1.731 MB    │ 1.731 MB      │ 1.731 MB      │ 1.731 MB      │         │
│                       shrink:       │               │               │               │         │
│                         15          │ 15            │ 15            │ 15            │         │
│                         1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ medium_project       7.262 ms      │ 19.89 ms      │ 7.551 ms      │ 7.801 ms      │ 100     │ 100
│                       max alloc:    │               │               │               │         │
│                         8564        │ 8564          │ 8564          │ 8564          │         │
│                         711.8 KB    │ 711.8 KB      │ 711.8 KB      │ 711.8 KB      │         │
│                       alloc:        │               │               │               │         │
│                         19559       │ 19559         │ 19559         │ 19559         │         │
│                         1.016 MB    │ 1.016 MB      │ 1.016 MB      │ 1.016 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         24881       │ 24881         │ 24881         │ 24881         │         │
│                         2.004 MB    │ 2.004 MB      │ 2.004 MB      │ 2.004 MB      │         │
│                       grow:         │               │               │               │         │
│                         4405        │ 4405          │ 4405          │ 4405          │         │
│                         705.4 KB    │ 705.4 KB      │ 705.4 KB      │ 705.4 KB      │         │
│                       shrink:       │               │               │               │         │
│                         15          │ 15            │ 15            │ 15            │         │
│                         1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
╰─ small_project        1.259 ms      │ 1.82 ms       │ 1.399 ms      │ 1.433 ms      │ 100     │ 100
                        max alloc:    │               │               │               │         │
                          579         │ 579           │ 579           │ 579           │         │
                          62.02 KB    │ 62.02 KB      │ 62.02 KB      │ 62.02 KB      │         │
                        alloc:        │               │               │               │         │
                          2654        │ 2654          │ 2654          │ 2654          │         │
                          160.2 KB    │ 160.2 KB      │ 160.2 KB      │ 160.2 KB      │         │
                        dealloc:      │               │               │               │         │
                          3086        │ 3086          │ 3086          │ 3086          │         │
                          246.2 KB    │ 246.2 KB      │ 246.2 KB      │ 246.2 KB      │         │
                        grow:         │               │               │               │         │
                          869         │ 869           │ 869           │ 869           │         │
                          69.52 KB    │ 69.52 KB      │ 69.52 KB      │ 69.52 KB      │         │
                        shrink:       │               │               │               │         │
                          15          │ 15            │ 15            │ 15            │         │
                          1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
```

</details>

## Summary

| Benchmark | Median Time |
|-----------|-------------|
| extra_large_project | 60.44 ms |
| large_project | 28.62 ms |
| medium_project | 7.551 ms |
| small_project | 1.399 ms |
