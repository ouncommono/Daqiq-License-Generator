# tests/test_visible_launch.ps1
# Gate 6: Real Flagless Visible Launch Test for Daqiq License Studio

$ErrorActionPreference = "Stop"

$exePath = Resolve-Path "target\release\daqiq-license-generator.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "Release executable not found at: $exePath"
    exit 1
}

$logPath = Join-Path $env:LOCALAPPDATA "DaqiqLicenseGenerator\startup.log"
if (Test-Path $logPath) {
    Remove-Item -Force $logPath
}

Write-Output "=== [GATE 6] Starting Real Flagless Visible Launch Test ==="
Write-Output "Launching binary with zero flags: $exePath"

# Add Win32 signatures for window visibility and dimensions check
Add-Type @"
using System;
using System.Runtime.InteropServices;

public class Win32Gui {
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    private struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    public static int[] GetWindowBounds(IntPtr hWnd) {
        RECT r;
        if (GetWindowRect(hWnd, out r)) {
            return new int[] { r.Right - r.Left, r.Bottom - r.Top, r.Left, r.Top };
        }
        return new int[] { 0, 0, 0, 0 };
    }

    public static IntPtr FindProcessMainWindow(int targetPid) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((hWnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hWnd, out pid);
            if (pid == (uint)targetPid && IsWindowVisible(hWnd)) {
                var bounds = GetWindowBounds(hWnd);
                if (bounds[0] >= 400 && bounds[1] >= 300) {
                    found = hWnd;
                    return false;
                }
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@

$proc = Start-Process -FilePath $exePath -PassThru
if (-not $proc -or $proc.HasExited) {
    Write-Error "FAIL: Process exited immediately or failed to start!"
    exit 1
}

Write-Output "Process started with PID: $($proc.Id)"

$hwnd = [IntPtr]::Zero
$timeoutSeconds = 12
$elapsed = 0

while ($elapsed -lt $timeoutSeconds) {
    Start-Sleep -Milliseconds 500
    $elapsed += 0.5
    $proc.Refresh()
    if ($proc.HasExited) {
        Write-Error "FAIL: Process unexpectedly died during startup (ExitCode: $($proc.ExitCode))!"
        if (Test-Path $logPath) {
            Write-Output "--- startup.log output ---"
            Get-Content $logPath
        }
        exit 1
    }

    $candidateHwnd = [Win32Gui]::FindProcessMainWindow($proc.Id)
    if ($candidateHwnd -ne [IntPtr]::Zero) {
        $hwnd = $candidateHwnd
    } elseif ($proc.MainWindowHandle -ne [IntPtr]::Zero) {
        $hwnd = $proc.MainWindowHandle
    }

    if ($hwnd -ne [IntPtr]::Zero) {
        if (Test-Path $logPath) {
            $currentLog = Get-Content $logPath -Raw -ErrorAction SilentlyContinue
            if ($currentLog -and $currentLog -match "\[ENTER_MSG_LOOP\]") {
                break
            }
        }
    }
}

if ($hwnd -eq [IntPtr]::Zero) {
    Write-Error "FAIL: MainWindowHandle remained 0 after $timeoutSeconds seconds!"
    Stop-Process -Id $proc.Id -Force
    exit 1
}

Write-Output "Discovered MainWindowHandle: 0x$($hwnd.ToString("X"))"

# Verify Window Visibility
$isVisible = [Win32Gui]::IsWindowVisible($hwnd)
Write-Output "IsWindowVisible: $isVisible"
if (-not $isVisible) {
    Write-Error "FAIL: Main window handle exists but IsWindowVisible is false!"
    Stop-Process -Id $proc.Id -Force
    exit 1
}

# Verify Window Dimensions
$bounds = [Win32Gui]::GetWindowBounds($hwnd)
$w = $bounds[0]
$h = $bounds[1]
$left = $bounds[2]
$top = $bounds[3]
Write-Output "Window Geometry: Width = $w, Height = $h (Left=$left, Top=$top)"

if ($w -lt 400 -or $h -lt 300) {
    Write-Error "FAIL: Window dimensions ($w x $h) are smaller than minimum allowed (400 x 300)!"
    Stop-Process -Id $proc.Id -Force
    exit 1
}

# Verify startup.log contains all critical breadcrumbs
if (-not (Test-Path $logPath)) {
    Write-Error "FAIL: startup.log was not created at: $logPath"
    Stop-Process -Id $proc.Id -Force
    exit 1
}

$logContent = Get-Content $logPath -Raw
Write-Output "--- Discovered startup.log Breadcrumbs ---"
Write-Output $logContent

$requiredStages = @(
    "BOOT_START",
    "PANIC_HOOK_OK",
    "DATA_DIR_OK",
    "CONFIG_OK",
    "DB_OK",
    "COM_OK",
    "COMMCTRL_OK",
    "FONT_OK",
    "CLASS_REG_OK",
    "MAIN_HWND_OK",
    "PANELS_CREATED_OK",
    "LAYOUT_OK",
    "SHOW_WINDOW_OK",
    "ENTER_MSG_LOOP"
)

foreach ($stage in $requiredStages) {
    if ($logContent -notmatch "\[$stage\]") {
        Write-Error "FAIL: Required stage [$stage] missing from startup.log!"
        Stop-Process -Id $proc.Id -Force
        exit 1
    }
}
Write-Output "PASS: All 14 startup lifecycle stages recorded in startup.log!"

# Test Graceful Window Close
Write-Output "Closing main window gracefully via PostMessage WM_CLOSE and CloseMainWindow()..."
[Win32Gui]::PostMessage($hwnd, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
$proc.CloseMainWindow() | Out-Null

$closed = $proc.WaitForExit(6000)
if (-not $closed) {
    Write-Error "FAIL: Process did not exit within 6 seconds of CloseMainWindow()!"
    Stop-Process -Id $proc.Id -Force
    exit 1
}

Write-Output "Process exited gracefully with ExitCode: $($proc.ExitCode)"
if ($proc.ExitCode -ne 0) {
    Write-Error "FAIL: Process exit code is $($proc.ExitCode), expected 0!"
    exit 1
}

Write-Output "=== [GATE 6] REAL FLAGLESS VISIBLE LAUNCH TEST PASSED 100% ==="
exit 0
