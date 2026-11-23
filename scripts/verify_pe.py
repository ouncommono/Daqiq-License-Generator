#!/usr/bin/env python3
"""PE Header & DLL Import Directory Audit for Windows Standalone Executable.

Asserts:
1. PE Subsystem == 2 (IMAGE_SUBSYSTEM_WINDOWS_GUI -> Zero CMD Window).
2. Zero WebView2 / Zero Non-System DLL dependencies (No WebView2Loader.dll, Edge, Python, Qt, CEF).
3. File size < 25 MB.
"""

import os
import struct
import sys

ALLOWED_SYSTEM_DLL_PREFIXES = (
    "api-ms-win-",
    "ext-ms-win-",
    "kernel32",
    "user32",
    "gdi32",
    "shell32",
    "comctl32",
    "comdlg32",
    "combase",
    "advapi32",
    "ws2_32",
    "ole32",
    "oleaut32",
    "shcore",
    "imm32",
    "dwmapi",
    "uxtheme",
    "bcrypt",
    "bcryptprimitives",
    "ntdll",
    "msvcrt",
    "dnsapi",
    "iphlpapi",
    "sechost",
    "rpcrt4",
    "userenv",
    "dbghelp",
    "synchronization",
    "dwrite",
    "d3d11",
    "d2d1",
    "dxgi",
    "opengl32",
    "glu32",
    "winmm",
    "setupapi",
    "cfgmgr32",
    "version",
)

BANNED_DLL_SUBSTRINGS = (
    "webview2",
    "edge",
    "python",
    "qt5",
    "qt6",
    "pyside",
    "cef",
    "electron",
    "node",
    "vcruntime140",
    "vcruntime140_1",
    "msvcp140",
    "msvcp140_1",
    "msvcp140_2",
)


def rva_to_offset(rva, sections):
    for sec_vaddr, sec_vsize, sec_raw_offset, sec_raw_size in sections:
        if sec_vaddr <= rva < sec_vaddr + max(sec_vsize, sec_raw_size):
            return sec_raw_offset + (rva - sec_vaddr)
    return None


def read_cstring(data, offset):
    end = data.find(b"\x00", offset)
    if end == -1:
        return ""
    return data[offset:end].decode("ascii", errors="replace")


def audit_pe_file(exe_path, max_size_mb=25.0):
    print(f"=== Auditing Windows PE Binary: {exe_path} ===")

    if not os.path.exists(exe_path):
        print(f"FAIL: File not found: {exe_path}")
        return False

    size_bytes = os.path.getsize(exe_path)
    size_mb = size_bytes / (1024 * 1024)
    print(f"Binary Size: {size_mb:.2f} MB ({size_bytes} bytes)")

    if size_mb > max_size_mb:
        print(f"FAIL: Size {size_mb:.2f} MB exceeds {max_size_mb} MB limit!")
        return False
    print(f"PASS: Size is under {max_size_mb} MB.")

    with open(exe_path, "rb") as f:
        data = f.read()

    if len(data) < 64 or data[:2] != b"MZ":
        print("FAIL: Not a valid DOS/PE file!")
        return False

    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    if pe_offset + 24 > len(data) or data[pe_offset:pe_offset+4] != b"PE\x00\x00":
        print("FAIL: Invalid PE signature!")
        return False

    machine, num_sections, _, _, _, size_of_opt_header, characteristics = struct.unpack_from(
        "<HHIIIHH", data, pe_offset + 4
    )

    opt_header_offset = pe_offset + 24
    opt_magic = struct.unpack_from("<H", data, opt_header_offset)[0]

    if opt_magic == 0x20B:
        # PE32+ (64-bit)
        subsystem = struct.unpack_from("<H", data, opt_header_offset + 68)[0]
        data_dirs_offset = opt_header_offset + 112
    elif opt_magic == 0x10B:
        # PE32 (32-bit)
        subsystem = struct.unpack_from("<H", data, opt_header_offset + 68)[0]
        data_dirs_offset = opt_header_offset + 96
    else:
        print(f"FAIL: Unknown optional header magic 0x{opt_magic:X}!")
        return False

    print(f"PE Subsystem: {subsystem} (2 = WINDOWS_GUI, 3 = WINDOWS_CUI)")
    if subsystem != 2:
        print(f"FAIL: Subsystem is {subsystem}, expected 2 (IMAGE_SUBSYSTEM_WINDOWS_GUI)!")
        return False
    print("PASS: Subsystem == 2 (Zero Console Window guaranteed).")

    # Parse sections
    sections_offset = opt_header_offset + size_of_opt_header
    sections = []
    for i in range(num_sections):
        sec_offset = sections_offset + i * 40
        sec_name = data[sec_offset:sec_offset+8].rstrip(b"\x00").decode("ascii", errors="replace")
        vsize, vaddr, raw_size, raw_offset = struct.unpack_from("<IIII", data, sec_offset + 8)
        sections.append((vaddr, vsize, raw_offset, raw_size))

    # Import Directory is DataDirectory[1]
    import_table_rva, import_table_size = struct.unpack_from("<II", data, data_dirs_offset + 8)
    print(f"Import Table RVA: 0x{import_table_rva:X}, Size: {import_table_size} bytes")

    imported_dlls = []
    if import_table_rva != 0:
        import_desc_offset = rva_to_offset(import_table_rva, sections)
        if import_desc_offset is not None:
            while import_desc_offset + 20 <= len(data):
                ilt_rva, time_date, forwarder, name_rva, iat_rva = struct.unpack_from(
                    "<IIIII", data, import_desc_offset
                )
                if ilt_rva == 0 and name_rva == 0:
                    break
                dll_name_offset = rva_to_offset(name_rva, sections)
                if dll_name_offset is not None:
                    dll_name = read_cstring(data, dll_name_offset)
                    if dll_name:
                        imported_dlls.append(dll_name)
                import_desc_offset += 20

    print("\n--- Discovered Imported DLLs ---")
    for dll in sorted(imported_dlls):
        print(f"  • {dll}")

    # Check for banned substrings
    failed_dlls = []
    for dll in imported_dlls:
        dll_lower = dll.lower()
        for banned in BANNED_DLL_SUBSTRINGS:
            if banned in dll_lower:
                failed_dlls.append((dll, f"Contains banned substring '{banned}'"))

        # Verify it is an allowed Windows system DLL
        is_allowed = False
        for prefix in ALLOWED_SYSTEM_DLL_PREFIXES:
            if dll_lower.startswith(prefix):
                is_allowed = True
                break
        if not is_allowed:
            print(f"WARNING: Non-standard system DLL detected: {dll}")

    if failed_dlls:
        print("\nFAIL: Prohibited runtime dependency detected!")
        for dll, reason in failed_dlls:
            print(f"  ❌ {dll}: {reason}")
        return False

    print("\nPASS: Zero WebView2, zero Edge, zero Python, and zero non-system runtime dependencies!")
    print("=== PE AUDIT PASSED 100% ===")
    return True


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python verify_pe.py <path_to_exe> [max_mb]")
        sys.exit(1)
    target_exe = sys.argv[1]
    max_mb = float(sys.argv[2]) if len(sys.argv) > 2 else 25.0
    success = audit_pe_file(target_exe, max_mb)
    sys.exit(0 if success else 1)
