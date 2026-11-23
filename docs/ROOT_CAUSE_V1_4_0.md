# Forensic Investigation & Root Cause Analysis: Daqiq License Generator v1.4.0 Launch Failure

## Executive Summary
In release **v1.4.0**, the compiled Windows executable failed to launch cleanly on standard/clean Windows desktop environments (exiting silently before window creation, hanging, or failing to render).

This forensic report investigates the startup sequence and Win32 window lifecycle across all 10 classic Windows desktop failure vectors, identifies the primary and contributing root causes, explains why CI passed despite these bugs, and details the architectural remedies implemented in **v1.4.1**.

---

## 1. Primary Root Cause: Dynamic MSVC C-Runtime (`VCRUNTIME140.dll` / `MSVCP140.dll`)

### Finding
In v1.4.0, the GitHub Actions CI workflow compiled the release executable using the `stable-x86_64-pc-windows-msvc` toolchain with default Cargo compiler flags (`cargo build --release -p daqiq-license-generator`).
By default, the MSVC target dynamically links the Microsoft Visual C++ Runtime (`VCRUNTIME140.dll`, `MSVCP140.dll`, `vcruntime140_1.dll`).

### Mechanism of Failure
When a user on a clean, air-gapped, or newly provisioned Windows 10/11 machine (without Visual Studio or the VC++ Redistributable installed) runs the executable:
1. The Windows OS kernel image loader (`ntdll.dll!LdrpProcessInitialization`) parses the PE Import Directory.
2. It attempts to resolve `VCRUNTIME140.dll`.
3. Finding no such DLL in `%SystemRoot%\System32` or application directory, the loader aborts process execution immediately with status code `0xC0000135` (`STATUS_DLL_NOT_FOUND`).
4. Because the PE header subsystem is set to `2` (`IMAGE_SUBSYSTEM_WINDOWS_GUI`), no console window is attached; the process dies instantly and completely silently.
5. Not a single line of application Rust code (including `main()` or the panic hook) is executed.

### Why CI Missed It in v1.4.0
- GitHub Actions `windows-latest` runner virtual machines have Visual Studio 2022 Community pre-installed with the C++ desktop workload, which places `VCRUNTIME140.dll` directly in the system PATH.
- `scripts/verify_pe.py` in v1.4.0 explicitly listed `vcruntime140`, `vcruntime140_1`, and `msvcp140` in `ALLOWED_SYSTEM_DLL_PREFIXES`.
- CI Gates 4 and 5 ran with `--self-test` flags inside the VS-equipped runner, succeeding without detecting the missing redistribution dependency.

### Resolution in v1.4.1
- Configure static CRT in `.cargo/config.toml`:
  ```toml
  [target.x86_64-pc-windows-msvc]
  rustflags = ["-C", "target-feature=+crt-static"]
  ```
- Remove `vcruntime140`, `vcruntime140_1`, and `msvcp140` from `ALLOWED_SYSTEM_DLL_PREFIXES` in `scripts/verify_pe.py`, actively forbidding dynamic VC++ runtime dependencies.
- Embed the CRT directly into the PE binary, ensuring 100% self-contained execution with zero external installer dependencies.

---

## 2. Contributing Vector: Inverted Extended Window Styles (`dwExStyle` vs `dwStyle`)

### Finding
In `apps/daqiq-lic-gui/src/main.rs`:
```rust
let hwnd = CreateWindowExW(
    WS_CLIPCHILDREN | WS_CLIPSIBLINGS, // Arg 1: dwExStyle (INCORRECT!)
    class_name.as_ptr(),
    title.as_ptr(),
    WS_OVERLAPPEDWINDOW | WS_VISIBLE,   // Arg 4: dwStyle
    ...
);
```
`WS_CLIPCHILDREN` (`0x02000000`) and `WS_CLIPSIBLINGS` (`0x04000000`) are standard window styles (`WS_`), NOT extended window styles (`WS_EX_`).
In extended window styles, `0x02000000` is `WS_EX_COMPOSITED`, which forces all descendants to be painted double-buffered bottom-up using GDI redirection, which can trigger severe painting glitches, black screens, or failures on certain Windows display drivers.
Furthermore, `WS_CLIPCHILDREN | WS_CLIPSIBLINGS` were missing from `dwStyle`, allowing child controls to paint over parent windows without proper clipping.

### Resolution in v1.4.1
- Set `dwExStyle` to `WS_EX_APPWINDOW | WS_EX_CONTROLPARENT` (or `0`).
- Combine `WS_CLIPCHILDREN | WS_CLIPSIBLINGS` into `dwStyle`:
  ```rust
  dwStyle = WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS;
  ```

---

## 3. Contributing Vector: Null `hInstance` in Window Registration & Creation

### Finding
`wnd_class.hInstance` was left as `0 as _` (NULL), and `CreateWindowExW` passed `0 as _` for `hInstance`.
On standard Win32, module instances should be resolved via `GetModuleHandleW(std::ptr::null())` so that the window class is registered under the process instance and child controls are bound to the actual module.

### Resolution in v1.4.1
- Obtain `hinstance = GetModuleHandleW(std::ptr::null())`.
- Assign `wnd_class.hInstance = hinstance`.
- Pass `hinstance` to `CreateWindowExW` and child control creations.

---

## 4. Contributing Vector: Missing COM & Common Controls Classes

### Finding
- COM library was not initialized (`CoInitializeEx` was absent), risking failures in shell dialogues and DPI scaling handlers.
- `InitCommonControlsEx` only requested `ICC_WIN95_CLASSES | ICC_STANDARD_CLASSES | ICC_LISTVIEW_CLASSES`, omitting `ICC_TAB_CLASSES` and `ICC_BAR_CLASSES`.

### Resolution in v1.4.1
- Call `CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED)`.
- Request full control classes: `ICC_WIN95_CLASSES | ICC_STANDARD_CLASSES | ICC_LISTVIEW_CLASSES | ICC_TAB_CLASSES | ICC_BAR_CLASSES`.

---

## 5. Contributing Vector: Absence of Startup Breadcrumb Logging & Error Reporting

### Finding
Because the subsystem is `IMAGE_SUBSYSTEM_WINDOWS_GUI`, stdout/stderr are suppressed. If any error occurred during initialization, no timestamped trace was recorded, leaving the user with a silent termination.

### Resolution in v1.4.1
- Implement dedicated `startup.log` breadcrumb tracing:
  Every phase records its timestamp and status in `%LOCALAPPDATA%\DaqiqLicenseGenerator\startup.log`:
  ```
  BOOT_START -> PANIC_HOOK_OK -> DATA_DIR_OK -> CONFIG_OK -> DB_OK -> COM_OK -> COMMCTRL_OK -> FONT_OK -> CLASS_REG_OK -> MAIN_HWND_OK -> PANELS_CREATED_OK -> LAYOUT_OK -> SHOW_WINDOW_OK -> ENTER_MSG_LOOP
  ```
- Catch fatal errors and display an informative native Win32 `MessageBoxW` with technical details and `GetLastError()`.

---

## 6. Contributing Vector: Database & Config Storage Robustness

### Finding
`AppState::new()` called `RegistryDb::open_in_memory()`, failing to persist SQLite records to `%LOCALAPPDATA%`. If a file path was used, it defaulted to the working directory without fallback or error recovery.

### Resolution in v1.4.1
- Explicitly target `%LOCALAPPDATA%\DaqiqLicenseGenerator\registry.sqlite`.
- If the existing database is corrupted, back it up to `.corrupt.bak` and re-initialize cleanly.
- Ensure `config.json` uses default fallbacks on deserialization errors.

---

## 7. Contributing Vector: Window Geometry & DWM Attributes

### Finding
Window coordinates were hardcoded to `(100, 100)` with no consideration of multi-monitor setups or work area bounds (`SPI_GETWORKAREA`).
`DwmSetWindowAttribute` used attribute `20` (`DWMWA_USE_IMMERSIVE_DARK_MODE`) unconditionally, which fails on Windows 10 builds prior to 2004 (build 19041).

### Resolution in v1.4.1
- Center the main window dynamically within `SystemParametersInfoW(SPI_GETWORKAREA, ...)`.
- Query `DwmSetWindowAttribute(hwnd, 20, ...)`; on failure, fall back to attribute `19` for Windows 10 1809/1903 compatibility.

---

## Summary Matrix

| Failure Vector | v1.4.0 Defect | v1.4.1 Permanent Remedy |
| :--- | :--- | :--- |
| **MSVC CRT** | Dynamic linkage (`VCRUNTIME140.dll`) | `-C target-feature=+crt-static` via `.cargo/config.toml` |
| **PE Audit** | Allowed `vcruntime140` | Banned all VC++ runtime DLLs from import directory |
| **Window Styles** | `WS_CLIPCHILDREN` in `dwExStyle` | Proper `dwStyle` / `dwExStyle` separation |
| **Module Handle** | NULL `hInstance` | `GetModuleHandleW(null())` passed throughout |
| **COM Subsystem** | Not initialized | `CoInitializeEx(APARTMENTTHREADED)` at startup |
| **Common Controls** | Missing tab/bar classes | Full class flags in `InitCommonControlsEx` |
| **Startup Tracing** | Zero logs on disk | Timestamped `%LOCALAPPDATA%\...\startup.log` breadcrumbs |
| **Crash Alert** | Silent die on early errors | Native `MessageBoxW` with `GetLastError()` |
| **Database Path** | Volatile in-memory fallback | Persistent `%LOCALAPPDATA%\...\registry.sqlite` with safe migration |
| **Window Layout** | Hardcoded (100, 100) | Dynamically centered in `SPI_GETWORKAREA` |
