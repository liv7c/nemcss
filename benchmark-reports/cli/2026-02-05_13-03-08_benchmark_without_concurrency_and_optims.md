# Benchmark Report: cli

**Date:** 2026-02-05 13:03:08 CET
**Git Branch:** `feat/cli/add_benchmarks_and_concurrency_build_command`
**Git Commit:** `5dcb76c (dirty)`
**Description:** benchmark_without_concurrency_and_optims

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
| extra_large_project | 138 ms | 141.1 ms | 142.5 ms | 235.5 ms |
| large_project | 67.28 ms | 69.52 ms | 69.77 ms | 77.93 ms |
| medium_project | 16.13 ms | 16.39 ms | 16.46 ms | 17.63 ms |
| small_project | 2.383 ms | 2.442 ms | 2.477 ms | 2.862 ms |

<details>
<summary>Full Benchmark Details (click to expand)</summary>

```
Timer precision: 41 ns
build_command           fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ extra_large_project  138 ms        │ 235.5 ms      │ 141.1 ms      │ 142.5 ms      │ 100     │ 100
│                       max alloc:    │               │               │               │         │
│                         25625       │ 25625         │ 25625         │ 25633         │         │
│                         1.887 MB    │ 1.887 MB      │ 1.887 MB      │ 1.896 MB      │         │
│                       alloc:        │               │               │               │         │
│                         198498      │ 198498        │ 198498        │ 198562        │         │
│                         14.03 MB    │ 14.03 MB      │ 14.03 MB      │ 14.05 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         198502      │ 198502        │ 198502        │ 198558        │         │
│                         15.96 MB    │ 15.96 MB      │ 15.96 MB      │ 15.99 MB      │         │
│                       grow:         │               │               │               │         │
│                         14650       │ 14650         │ 14650         │ 14657         │         │
│                         1.934 MB    │ 1.934 MB      │ 1.934 MB      │ 1.949 MB      │         │
│                       shrink:       │               │               │               │         │
│                         15          │ 15            │ 15            │ 15.63         │         │
│                         1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 5.987 KB      │         │
├─ large_project        67.28 ms      │ 77.93 ms      │ 69.52 ms      │ 69.77 ms      │ 100     │ 100
│                       max alloc:    │               │               │               │         │
│                         24425       │ 24425         │ 24425         │ 24425         │         │
│                         1.764 MB    │ 1.764 MB      │ 1.764 MB      │ 1.764 MB      │         │
│                       alloc:        │               │               │               │         │
│                         130532      │ 130532        │ 130532        │ 130532        │         │
│                         10.57 MB    │ 10.57 MB      │ 10.57 MB      │ 10.57 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         130536      │ 130536        │ 130536        │ 130536        │         │
│                         12.3 MB     │ 12.3 MB       │ 12.3 MB       │ 12.3 MB       │         │
│                       grow:         │               │               │               │         │
│                         11141       │ 11141         │ 11141         │ 11141         │         │
│                         1.731 MB    │ 1.731 MB      │ 1.731 MB      │ 1.731 MB      │         │
│                       shrink:       │               │               │               │         │
│                         15          │ 15            │ 15            │ 15            │         │
│                         1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
├─ medium_project       16.13 ms      │ 17.63 ms      │ 16.39 ms      │ 16.46 ms      │ 100     │ 100
│                       max alloc:    │               │               │               │         │
│                         13781       │ 13781         │ 13781         │ 13781         │         │
│                         881.8 KB    │ 881.8 KB      │ 881.8 KB      │ 881.8 KB      │         │
│                       alloc:        │               │               │               │         │
│                         35005       │ 35005         │ 35005         │ 35005         │         │
│                         2.211 MB    │ 2.211 MB      │ 2.211 MB      │ 2.211 MB      │         │
│                       dealloc:      │               │               │               │         │
│                         35009       │ 35009         │ 35009         │ 35009         │         │
│                         2.915 MB    │ 2.915 MB      │ 2.915 MB      │ 2.915 MB      │         │
│                       grow:         │               │               │               │         │
│                         4405        │ 4405          │ 4405          │ 4405          │         │
│                         705.4 KB    │ 705.4 KB      │ 705.4 KB      │ 705.4 KB      │         │
│                       shrink:       │               │               │               │         │
│                         15          │ 15            │ 15            │ 15            │         │
│                         1.98 KB     │ 1.98 KB       │ 1.98 KB       │ 1.98 KB       │         │
╰─ small_project        2.383 ms      │ 2.862 ms      │ 2.442 ms      │ 2.477 ms      │ 100     │ 100
                        max alloc:    │               │               │               │         │
                          972         │ 972           │ 972           │ 972           │         │
                          62.02 KB    │ 62.02 KB      │ 62.02 KB      │ 62.02 KB      │         │
                        alloc:        │               │               │               │         │
                          3464        │ 3464          │ 3464          │ 3464          │         │
                          222.7 KB    │ 222.7 KB      │ 222.7 KB      │ 222.7 KB      │         │
                        dealloc:      │               │               │               │         │
                          3468        │ 3468          │ 3468          │ 3468          │         │
                          290.6 KB    │ 290.6 KB      │ 290.6 KB      │ 290.6 KB      │         │
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
| extra_large_project | 141.1 ms |
| large_project | 69.52 ms |
| medium_project | 16.39 ms |
| small_project | 2.442 ms |
