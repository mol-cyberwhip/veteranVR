# Agentic Appium Test Harness

Automated end-to-end testing harness for the Rookie Desktop macOS application using Appium with comprehensive debugging capabilities.

## Overview

This harness provides **fully automated** macOS UI testing with:
- Automatic Appium server lifecycle management (start/stop)
- Screenshot capture at each test step
- Page source (accessibility tree) capture
- Comprehensive logging
- Automatic cleanup on success or failure

## Quick Start

### Prerequisites

- macOS with full Xcode installed
- Node.js and Appium with mac2 driver
- Python 3 with the project virtual environment activated

### Basic Usage

```bash
# Run the test with default settings
python test_tauri_driving/agentic_macos_e2e.py
```

This will:
1. Check Xcode installation
2. Build the app if needed
3. Start Appium server automatically
4. Run the E2E test
5. Capture screenshots and page sources
6. Stop Appium server and clean up

### Output Location

All test artifacts are saved to `./harness_runs/run_YYYYMMDD_HHMMSS/`:

```
harness_runs/run_20260212_202405/
├── test-log.json           # Structured test execution log
├── appium-server.log       # Full Appium server output
├── screenshots/
│   ├── 202405_01_initial.png
│   ├── 202405_02_bootstrap_detected.png
│   └── ...
└── page-source/
    ├── 202405_01_initial.xml
    └── ...
```

## Command-Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--appium-port` | 4725 | Port for Appium server |
| `--force-build` | False | Rebuild the app before testing |
| `--session-timeout-seconds` | 120.0 | Timeout for session creation |
| `--server-startup-timeout-ms` | 120000 | Timeout for WebDriverAgentMac |
| `--log-level` | debug | Appium log level (debug/info/warn/error) |
| `--artifacts-dir` | auto | Custom directory for artifacts |

### Examples

**Force rebuild the app:**
```bash
python test_tauri_driving/agentic_macos_e2e.py --force-build
```

**Use debug logging:**
```bash
python test_tauri_driving/agentic_macos_e2e.py --log-level debug
```

**Custom artifacts directory:**
```bash
python test_tauri_driving/agentic_macos_e2e.py --artifacts-dir /tmp/my-test-run
```

**Shorter timeouts for faster feedback:**
```bash
python test_tauri_driving/agentic_macos_e2e.py \
  --session-timeout-seconds 30 \
  --server-startup-timeout-ms 30000
```

## Test Steps

The harness executes the following automated steps:

1. **check_xcode** - Verify full Xcode is installed
2. **build_app** - Build the app bundle if needed
3. **start_server** - Start Appium server with mac2 driver
4. **create_session** - Create Appium session for the app
5. **capture_initial_state** - Take initial screenshot and page source
6. **wait_for_bootstrap** - Wait for "Rookie Desktop Bootstrap" text
7. **find_start_button** - Find the "Start Success Operation" button
8. **click_start_button** - Click the button
9. **wait_for_success** - Wait for "succeeded" state

## Artifacts Explained

### Screenshots (`screenshots/*.png`)
PNG images captured at key moments:
- `01_initial` - App just launched
- `02_bootstrap_detected` - Bootstrap UI visible
- `03_after_click` - After clicking start button
- `04_success_state` - Final success state
- `99_failure` - Only if test fails

### Page Source (`page-source/*.xml`)
XML files containing the accessibility hierarchy:
- All UI elements with attributes (labels, titles, values, etc.)
- Element positions and dimensions
- Complete tree structure
- Useful for debugging why elements weren't found

### Test Log (`test-log.json`)
Structured JSON log with:
- Timestamps for each step
- Step status (running/passed/failed)
- Error details if any
- Check results

### Appium Server Log (`appium-server.log`)
Complete Appium server output including:
- HTTP requests/responses
- WebDriverAgentMac build output
- Session lifecycle events
- Error messages

## Debugging Failed Tests

When a test fails, the harness:

1. Takes a screenshot of the failure state (`99_failure.png`)
2. Saves the page source at failure time (`99_failure.xml`)
3. Preserves all logs
4. Attempts to clean up resources
5. Outputs error details to console

### Debugging Checklist

1. **Check screenshots** - View `99_failure.png` to see the UI state
2. **Check page source** - Look at `99_failure.xml` for element details
3. **Check Appium logs** - Review `appium-server.log` for driver errors
4. **Check test log** - Review `test-log.json` for step-by-step execution

## Manual Appium Server (Alternative)

If you prefer to manage the Appium server manually, use the original harness:

```bash
# Terminal 1: Start Appium
appium --use-drivers=mac2 --port 4725

# Terminal 2: Run test (connects to existing server)
python test_tauri_driving/macos_appium_e2e.py --appium-url http://127.0.0.1:4725
```

## Troubleshooting

### "Port 4725 is already in use"
Either:
- Stop the existing Appium server
- Use a different port: `--appium-port 4726`

### Session timeout errors
Increase timeouts:
```bash
python test_tauri_driving/agentic_macos_e2e.py \
  --session-timeout-seconds 180 \
  --server-startup-timeout-ms 180000
```

### Xcode not found
Install full Xcode from the App Store and run:
```bash
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
```

### Screenshots show wrong window
Note: mac2 driver captures the entire screen, not just the app window. This is expected behavior.

## Requirements

- macOS 11 or later
- Full Xcode (not just Command Line Tools)
- Xcode Helper app enabled for Accessibility access
- `appium` and `appium-mac2-driver` installed globally
- Tauri CLI (`cargo tauri`)
- Python 3.11+ with project virtual environment

## Comparison with Original Harness

| Feature | Original `macos_appium_e2e.py` | Agentic `agentic_macos_e2e.py` |
|---------|-------------------------------|-------------------------------|
| Server management | Manual (external) | Automatic (self-contained) |
| Screenshots | No | Yes (PNG files) |
| Page source | No | Yes (XML files) |
| Artifacts directory | None | `./harness_runs/` |
| Failure debugging | Limited | Comprehensive |
| Ease of use | Requires 2 terminals | Single command |

## Exit Codes

- `0` - All tests passed
- `1` - Test failed or error occurred

## Notes

- The harness automatically creates `./harness_runs/` directory (gitignored)
- Press Ctrl+C to interrupt - cleanup will still run
- Screenshots capture the entire screen (mac2 driver limitation)
- Each run creates a new timestamped subdirectory
- Failed runs include `99_failure.*` artifacts for debugging
