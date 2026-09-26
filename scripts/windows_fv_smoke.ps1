[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Binary,

    # The pinned official Codex CLI (0.146.0-alpha.3.1) for the
    # "valid runtime" launches (FV-W03's relaunch leg). Optional: when
    # absent the driver searches the PATH, then falls back to the
    # harmless stub and records the named honest bound in the scene
    # notes (never fabricated).
    [string]$CodexBin = '',

    # Scene ids to run (default: every catalog Windows scene FV-W01..W12).
    [string[]]$Scene = @(),

    # Where captures + scene notes land (the workflow uploads this dir).
    [string]$EvidenceRoot = ''
)

# FV-002 — the Windows CI journey-smoke driver (the SendKeys scene
# driver for the FV catalog's Windows lane, §2; Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b — the chords are the app's
# registered bindings; every SendKeys chord below cites its source):
#   - Ctrl+N (new task): KeyBinding shortcut("n") NewChatShortcut —
#     crates/codex-app/src/ui.rs:5743 -> SendKeys "^n"
#   - Ctrl+K (palette): ui.rs:5739 -> "^k"
#   - Ctrl+/ (shortcuts overlay): KeyBinding shortcut("/") ShowKeyboard-
#     ShortcutsShortcut ui.rs:5779 -> "^/"
#   - Ctrl+P (files palette, the no-workspace guard):
#     files_palette_command_status "Select a workspace before searching
#     files." ui.rs:48202-48209 -> "^p"
#   - Ctrl+Alt+Shift+6 (capability panel): ui.rs:5809 -> "^%+6"
#   - Ctrl+Alt+Shift+M (model picker): ui.rs:5806 -> "^%+m"
#   - Ctrl+Alt+U (Activity view toggle): registry "toggleActivityView"
#     ui.rs:3327-3333 -> "^%u"
#   - Ctrl+Alt+4 (the Activity surface): ui.rs:5790 -> "^%4"
#   - Ctrl+Alt+Shift+S (save panel): ui.rs:5855 -> "^%+s"
#   - Ctrl+Alt+2 (Reusable workflows surface): ui.rs:5788 -> "^%2"
#   - Ctrl+Alt+Shift+U (members panel): ui.rs:5862 -> "^%+u"
#   - Ctrl+Alt+Shift+L (conflicts panel): ui.rs:5868 -> "^%+l"
#   - Ctrl+Alt+Shift+Y (needs-you panel): ui.rs:5874 -> "^%+y"
#   - Ctrl+Alt+Shift+7 (agents view): ui.rs:5853 -> "^%+7"
#   - Escape closes the panels (the scoped bindings, ui.rs:5882-5935)
#     -> "{ESC}"; Tab/Shift+Tab cycle -> "{TAB}"/"+{TAB}"
#   - the composer submit key is Ctrl+Return (the D11r FW-5 calibration:
#     plain Return does not submit) -> "^{ENTER}" (the catalog's "Enter"
#     moment is driven first per the scene row, then the calibrated
#     submit — both captured; honest notes)
#
# DEPTH LAW (WAVE7 addendum §2): SendKeys + window-state assertions
# only — process alive, main window handle present, clean
# CloseMainWindow exit. Captures via the .NET capture API where the
# runner session allows (honest WARN + skip otherwise — never
# fabricated). Everything beyond this depth is a named gap in the
# FV-CATALOG §5 list (W-1/W-2/W-5), never silently narrowed.
#
# Usage (CI wiring passes -Binary target/release/codexrs.exe):
#   pwsh scripts/windows_fv_smoke.ps1 -Binary <codexrs.exe> `
#        [-CodexBin <pinned codex cli>] [-Scene fv-w01,...] `
#        [-EvidenceRoot <dir>]

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
    throw "FV-002 windows journey-smoke: the binary is not a file: $Binary"
}

if ([string]::IsNullOrWhiteSpace($EvidenceRoot)) {
    $EvidenceRoot = Join-Path (Get-Location) 'fv-gate/windows'
}
$null = New-Item -ItemType Directory -Path $EvidenceRoot -Force

$script:StartedAtIso = [DateTime]::UtcNow.ToString('u')
$script:SceneResults = [System.Collections.Generic.List[object]]::new()

# ---------------------------------------------------------------------------
# Prerequisites (hard-fail with named messages — never silent)
# ---------------------------------------------------------------------------

$falseCodexBinary = Join-Path $env:SystemRoot 'System32\where.exe'
if (-not (Test-Path -LiteralPath $falseCodexBinary -PathType Leaf)) {
    throw "FV-002 windows journey-smoke: cannot find the harmless Codex stub: $falseCodexBinary"
}

$validCodexBinary = ''
if (-not [string]::IsNullOrWhiteSpace($CodexBin)) {
    if (Test-Path -LiteralPath $CodexBin -PathType Leaf) {
        $validCodexBinary = (Resolve-Path -LiteralPath $CodexBin).Path
    }
    else {
        Write-Warning "FV-002 windows journey-smoke: -CodexBin does not exist ($CodexBin) — the valid-runtime legs fall back to the harmless stub with the named honest bound"
    }
}
if ([string]::IsNullOrWhiteSpace($validCodexBinary)) {
    # Honest fallback: search the PATH for a codex CLI (the CI step may
    # have installed the pinned one without passing -CodexBin).
    $found = Get-Command codex -ErrorAction SilentlyContinue
    if ($null -ne $found) {
        $validCodexBinary = $found.Source
    }
}

try {
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing
    $script:CaptureAvailable = $true
}
catch {
    $script:CaptureAvailable = $false
    Write-Warning "FV-002 windows journey-smoke: the .NET capture API is unavailable in this session ($($_.Exception.Message)) — captures are skipped (honest absence; window-state assertions still run)"
}

try {
    Add-Type -AssemblyName Microsoft.VisualBasic
    $script:AppActivateAvailable = $true
}
catch {
    $script:AppActivateAvailable = $false
    Write-Warning "FV-002 windows journey-smoke: Microsoft.VisualBasic AppActivate is unavailable ($($_.Exception.Message)) — SendKeys rely on the runner's foreground window"
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function Write-SceneLog {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][string]$Message
    )
    $line = "[{0}] {1}" -f ([DateTime]::UtcNow.ToString('u')), $Message
    Write-Host $line
    Add-Content -LiteralPath (Join-Path $EvidenceRoot "$SceneId-notes.log") -Value $line
}

function Start-FlauzApp {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [string]$RuntimeBin = $falseCodexBinary,
        [string]$RuntimeLabel = 'stub (where.exe — the harmless precedent)'
    )
    $sceneRoot = Join-Path ([System.IO.Path]::GetTempPath()) "codexrs-fv-$SceneId-$([guid]::NewGuid().ToString('N'))"
    $codexHome = Join-Path $sceneRoot 'codex'
    $dataDirectory = Join-Path $sceneRoot 'data\codexrs'
    $null = New-Item -ItemType Directory -Path $codexHome -Force
    $null = New-Item -ItemType Directory -Path $dataDirectory -Force
    $stdoutLog = Join-Path $sceneRoot 'codexrs.stdout.log'
    $stderrLog = Join-Path $sceneRoot 'codexrs.stderr.log'

    $previousEnvironment = @{}
    foreach ($name in @('CODEX_HOME', 'CODEX_RS_DATA_DIR', 'CODEX_RS_CODEX_BIN')) {
        $previousEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    [Environment]::SetEnvironmentVariable('CODEX_HOME', $codexHome, 'Process')
    [Environment]::SetEnvironmentVariable('CODEX_RS_DATA_DIR', $dataDirectory, 'Process')
    [Environment]::SetEnvironmentVariable('CODEX_RS_CODEX_BIN', $RuntimeBin, 'Process')

    try {
        $process = Start-Process -FilePath $binaryPath -PassThru `
            -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog
    }
    finally {
        foreach ($name in $previousEnvironment.Keys) {
            [Environment]::SetEnvironmentVariable($name, $previousEnvironment[$name], 'Process')
        }
    }
    Write-SceneLog -SceneId $SceneId -Message "launched pid $($process.Id); runtime: $RuntimeLabel; isolated CODEX_HOME=$codexHome"
    [pscustomobject]@{
        Process   = $process
        SceneRoot = $sceneRoot
        StdoutLog = $stdoutLog
        StderrLog = $stderrLog
    }
}

function Wait-FlauzMainWindow {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][System.Diagnostics.Process]$Process,
        [int]$TimeoutSeconds = 30
    )
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        $Process.Refresh()
        if ($Process.HasExited) {
            throw "FV-002 $SceneId : the process exited during startup (status $($Process.ExitCode)) — the graceful-degradation contract failed"
        }
        if ($Process.MainWindowHandle -ne [IntPtr]::Zero) {
            Write-SceneLog -SceneId $SceneId -Message "main window present (handle $($Process.MainWindowHandle))"
            return $true
        }
        Start-Sleep -Milliseconds 500
    }
    throw "FV-002 $SceneId : the main window did not appear within $TimeoutSeconds s"
}

function Assert-FlauzAlive {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][System.Diagnostics.Process]$Process,
        [string]$Because = 'the drive'
    )
    $Process.Refresh()
    if ($Process.HasExited) {
        throw "FV-002 $SceneId : the process exited during $Because (status $($Process.ExitCode))"
    }
    if ($Process.MainWindowHandle -eq [IntPtr]::Zero) {
        throw "FV-002 $SceneId : the main window handle vanished after $Because"
    }
}

function Send-FlauzKeys {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][System.Diagnostics.Process]$Process,
        [Parameter(Mandatory = $true)][string]$Keys
    )
    $Process.Refresh()
    if ($script:AppActivateAvailable) {
        $null = [Microsoft.VisualBasic.Interaction]::AppActivate($Process.Id)
    }
    [System.Windows.Forms.SendKeys]::SendWait($Keys)
    Write-SceneLog -SceneId $SceneId -Message "SendKeys: $Keys"
}

function Escape-SendKeysText {
    param([Parameter(Mandatory = $true)][string]$Text)
    # SendKeys' reserved set: ^ % + ( ) { } [ ] ~ — wrap each in braces.
    ($Text.ToCharArray() | ForEach-Object {
            if ($_ -in @('^', '%', '+', '(', ')', '{', '}', '[', ']', '~')) {
                '{{{0}}}' -f $_
            }
            else {
                $_
            }
        }) -join ''
}

function Send-FlauzText {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][System.Diagnostics.Process]$Process,
        [Parameter(Mandatory = $true)][string]$Text
    )
    Send-FlauzKeys -SceneId $SceneId -Process $Process -Keys (Escape-SendKeysText $Text)
}

function Capture-FlauzFrame {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][string]$Name
    )
    if (-not $script:CaptureAvailable) {
        Write-SceneLog -SceneId $SceneId -Message "capture skipped ($Name) — the .NET capture API is unavailable in this session (honest absence)"
        return
    }
    try {
        $sceneDir = Join-Path $EvidenceRoot $SceneId
        $null = New-Item -ItemType Directory -Path $sceneDir -Force
        $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
        $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
                $path = Join-Path $sceneDir "$Name.png"
                $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
                Write-SceneLog -SceneId $SceneId -Message "capture: $Name.png"
            }
            finally {
                $graphics.Dispose()
            }
        }
        finally {
            $bitmap.Dispose()
        }
    }
    catch {
        Write-SceneLog -SceneId $SceneId -Message "capture failed ($Name): $($_.Exception.Message) — honest absence, window-state assertions still carry the scene"
    }
}

function Stop-FlauzApp {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][pscustomobject]$App,
        [bool]$ExpectCleanExit = $true
    )
    $process = $App.Process
    try {
        $process.Refresh()
        if (-not $process.HasExited) {
            if (-not $process.CloseMainWindow()) {
                throw "FV-002 $SceneId : CloseMainWindow did not accept the close request"
            }
            if (-not $process.WaitForExit(15000)) {
                throw "FV-002 $SceneId : the process did not exit within 15 s of the close request"
            }
            $process.Refresh()
            if ($ExpectCleanExit -and ($process.ExitCode -ne 0)) {
                throw "FV-002 $SceneId : the process exited with status $($process.ExitCode) on CloseMainWindow (clean exit expected)"
            }
            Write-SceneLog -SceneId $SceneId -Message "closed cleanly via CloseMainWindow (exit $($process.ExitCode))"
        }
    }
    finally {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
                $null = $process.WaitForExit(5000)
            }
        }
        finally {
            $process.Dispose()
            if (Test-Path -LiteralPath $App.SceneRoot) {
                # Keep the bounded app-log tails as scene evidence before
                # the temp cleanup.
                $sceneDir = Join-Path $EvidenceRoot $SceneId
                $null = New-Item -ItemType Directory -Path $sceneDir -Force
                foreach ($log in @($App.StdoutLog, $App.StderrLog)) {
                    if (Test-Path -LiteralPath $log -PathType Leaf) {
                        $tail = Get-Content -LiteralPath $log -Tail 200 -ErrorAction SilentlyContinue
                        if ($null -ne $tail) {
                            Add-Content -LiteralPath (Join-Path $sceneDir "$(Split-Path -Leaf $log).tail.txt") -Value $tail -ErrorAction SilentlyContinue
                        }
                    }
                }
                Remove-Item -LiteralPath $App.SceneRoot -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
    }
}

function Invoke-FlauzScene {
    param(
        [Parameter(Mandatory = $true)][string]$SceneId,
        [Parameter(Mandatory = $true)][scriptblock]$Body
    )
    Write-Host ""
    Write-Host "=== FV scene $SceneId ==="
    try {
        & $Body
        $script:SceneResults.Add([pscustomobject]@{ Scene = $SceneId; Verdict = 'pass' })
        Write-Host "FV scene $SceneId : PASS"
    }
    catch {
        $script:SceneResults.Add([pscustomobject]@{ Scene = $SceneId; Verdict = "fail: $($_.Exception.Message)" })
        Write-Warning "FV scene $SceneId : FAIL — $($_.Exception.Message)"
    }
}

# ---------------------------------------------------------------------------
# The catalog §2 scenes (FV-W01..FV-W12)
# ---------------------------------------------------------------------------

function Invoke-FvW01 {
    # J-01: launch -> main window present; SendKeys Ctrl+N -> the
    # new-task surface; type a short objective; Enter. No crash, clean
    # state at scene end.
    $app = Start-FlauzApp -SceneId 'fv-w01'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w01' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Assert-FlauzAlive -SceneId 'fv-w01' -Process $app.Process -Because 'startup settle'
        Capture-FlauzFrame -SceneId 'fv-w01' -Name '01-main-window'

        Send-FlauzKeys -SceneId 'fv-w01' -Process $app.Process -Keys '^n'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w01' -Process $app.Process -Because 'the Ctrl+N drive'
        Capture-FlauzFrame -SceneId 'fv-w01' -Name '02-new-task-surface'

        Send-FlauzText -SceneId 'fv-w01' -Process $app.Process -Text 'Plan the community garden layout for next spring'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w01' -Process $app.Process -Because 'typing the objective'
        Capture-FlauzFrame -SceneId 'fv-w01' -Name '03-objective-typed'

        # The catalog's "Enter" moment, then the calibrated submit key
        # (Ctrl+Return — the D11r FW-5 lesson: plain Return does not
        # submit). Both captured; the Lead adjudicates.
        Send-FlauzKeys -SceneId 'fv-w01' -Process $app.Process -Keys '{ENTER}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w01' -Name '04-after-enter'
        Send-FlauzKeys -SceneId 'fv-w01' -Process $app.Process -Keys '^{ENTER}'
        Start-Sleep -Seconds 5
        Assert-FlauzAlive -SceneId 'fv-w01' -Process $app.Process -Because 'the submit drive'
        Capture-FlauzFrame -SceneId 'fv-w01' -Name '05-after-submit'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w01' -App $app
    }
}

function Invoke-FvW02 {
    # J-02: type "/status" + Enter -> the composer status panel; Escape
    # closes. The window must stay responsive (the message pump alive —
    # the panel opens and closes).
    $app = Start-FlauzApp -SceneId 'fv-w02'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w02' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzText -SceneId 'fv-w02' -Process $app.Process -Text '/status'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w02' -Name '01-status-typed'
        Send-FlauzKeys -SceneId 'fv-w02' -Process $app.Process -Keys '{ENTER}'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w02' -Process $app.Process -Because 'the /status drive (the message pump must stay alive)'
        Capture-FlauzFrame -SceneId 'fv-w02' -Name '02-status-panel'
        Send-FlauzKeys -SceneId 'fv-w02' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w02' -Process $app.Process -Because 'the Escape close'
        Capture-FlauzFrame -SceneId 'fv-w02' -Name '03-escape-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w02' -App $app
    }
}

function Invoke-FvW03 {
    # J-03 (retryable startup): launch with CODEX_RS_CODEX_BIN pointing
    # at a missing binary -> the failure surface; the window stays up
    # >= 15 s; relaunch with a valid runtime -> the online footer.
    # Exit code 0 on clean CloseMainWindow (Stop-FlauzApp asserts).
    $missingRuntime = Join-Path ([System.IO.Path]::GetTempPath()) "codexrs-fv-missing-runtime-$([guid]::NewGuid().ToString('N')).exe"
    $app = Start-FlauzApp -SceneId 'fv-w03' -RuntimeBin $missingRuntime -RuntimeLabel 'MISSING binary (the graceful-degradation leg)'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w03' -Process $app.Process | Out-Null
        Capture-FlauzFrame -SceneId 'fv-w03' -Name '01-failure-surface'
        # The contract: >= 15 s alive under the missing runtime, no crash.
        Start-Sleep -Seconds 15
        Assert-FlauzAlive -SceneId 'fv-w03' -Process $app.Process -Because 'the 15 s missing-runtime watch (the graceful-degradation contract)'
        Capture-FlauzFrame -SceneId 'fv-w03' -Name '02-failure-surface-15s'
        Write-SceneLog -SceneId 'fv-w03' -Message 'graceful degradation held: process alive >= 15 s under the missing runtime'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w03' -App $app
    }

    # The relaunch with a valid runtime (the online footer).
    $runtimeForRelaunch = $validCodexBinary
    $runtimeLabel = if ([string]::IsNullOrWhiteSpace($runtimeForRelaunch)) {
        'stub fallback (the pinned Codex CLI is absent on this runner — the named honest bound; the online-footer frame is Lead-adjudicated from a run with the CLI installed)'
    }
    else {
        "valid runtime: $runtimeForRelaunch"
    }
    if ([string]::IsNullOrWhiteSpace($runtimeForRelaunch)) {
        $runtimeForRelaunch = $falseCodexBinary
    }
    $app2 = Start-FlauzApp -SceneId 'fv-w03' -RuntimeBin $runtimeForRelaunch -RuntimeLabel $runtimeLabel
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w03' -Process $app2.Process | Out-Null
        Start-Sleep -Seconds 10
        Capture-FlauzFrame -SceneId 'fv-w03' -Name '03-relaunch-state'
        Assert-FlauzAlive -SceneId 'fv-w03' -Process $app2.Process -Because 'the relaunch settle'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w03' -App $app2
    }
}

function Invoke-FvW04 {
    # J-04: Ctrl+Alt+Shift+6 -> the capability panel; Escape.
    $app = Start-FlauzApp -SceneId 'fv-w04'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w04' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w04' -Process $app.Process -Keys '^%+6'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w04' -Process $app.Process -Because 'the capability-panel drive'
        Capture-FlauzFrame -SceneId 'fv-w04' -Name '01-capability-panel-open'
        Send-FlauzKeys -SceneId 'fv-w04' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w04' -Process $app.Process -Because 'the scoped-Escape close'
        Capture-FlauzFrame -SceneId 'fv-w04' -Name '02-capability-panel-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w04' -App $app
    }
}

function Invoke-FvW05 {
    # J-14: Ctrl+Alt+Shift+M -> the model picker; Escape. The honest
    # empty state is visible in the capture.
    $app = Start-FlauzApp -SceneId 'fv-w05'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w05' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w05' -Process $app.Process -Keys '^%+m'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w05' -Process $app.Process -Because 'the model-picker drive'
        Capture-FlauzFrame -SceneId 'fv-w05' -Name '01-model-picker-open'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w05' -Name '02-model-picker-honest-empty'
        Send-FlauzKeys -SceneId 'fv-w05' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w05' -Process $app.Process -Because 'the Escape close'
        Capture-FlauzFrame -SceneId 'fv-w05' -Name '03-model-picker-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w05' -App $app
    }
}

function Invoke-FvW06 {
    # J-17: Ctrl+Alt+U -> the Activity view; Ctrl+Alt+4 -> the Activity
    # surface; Escape.
    $app = Start-FlauzApp -SceneId 'fv-w06'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w06' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w06' -Process $app.Process -Keys '^%u'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w06' -Process $app.Process -Because 'the Activity-view drive'
        Capture-FlauzFrame -SceneId 'fv-w06' -Name '01-activity-view-open'
        Send-FlauzKeys -SceneId 'fv-w06' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w06' -Name '02-activity-view-closed'
        Send-FlauzKeys -SceneId 'fv-w06' -Process $app.Process -Keys '^%4'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w06' -Process $app.Process -Because 'the Activity-surface drive'
        Capture-FlauzFrame -SceneId 'fv-w06' -Name '03-activity-surface-open'
        Send-FlauzKeys -SceneId 'fv-w06' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w06' -Process $app.Process -Because 'the Escape close'
        Capture-FlauzFrame -SceneId 'fv-w06' -Name '04-activity-surface-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w06' -App $app
    }
}

function Invoke-FvW07 {
    # J-10/J-11: Ctrl+Alt+Shift+S -> the save panel; Ctrl+Alt+2 -> the
    # Reusable workflows surface; Escape each. The honest
    # not-wired/empty states are visible.
    $app = Start-FlauzApp -SceneId 'fv-w07'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w07' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w07' -Process $app.Process -Keys '^%+s'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w07' -Process $app.Process -Because 'the save-panel drive'
        Capture-FlauzFrame -SceneId 'fv-w07' -Name '01-save-panel-open'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w07' -Name '02-save-panel-honest-state'
        Send-FlauzKeys -SceneId 'fv-w07' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w07' -Name '03-save-panel-closed'
        Send-FlauzKeys -SceneId 'fv-w07' -Process $app.Process -Keys '^%2'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w07' -Process $app.Process -Because 'the Reusable-workflows drive'
        Capture-FlauzFrame -SceneId 'fv-w07' -Name '04-reusable-workflows-open'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w07' -Name '05-reusable-workflows-empty'
        Send-FlauzKeys -SceneId 'fv-w07' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w07' -Process $app.Process -Because 'the Escape close'
        Capture-FlauzFrame -SceneId 'fv-w07' -Name '06-reusable-workflows-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w07' -App $app
    }
}

function Invoke-FvW08 {
    # J-13: Ctrl+Alt+Shift+U -> the members panel; Escape. The "Just
    # you" state is visible in the capture.
    $app = Start-FlauzApp -SceneId 'fv-w08'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w08' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w08' -Process $app.Process -Keys '^%+u'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w08' -Process $app.Process -Because 'the members-panel drive'
        Capture-FlauzFrame -SceneId 'fv-w08' -Name '01-members-panel-open'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w08' -Name '02-members-just-you'
        Send-FlauzKeys -SceneId 'fv-w08' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w08' -Process $app.Process -Because 'the Escape close'
        Capture-FlauzFrame -SceneId 'fv-w08' -Name '03-members-panel-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w08' -App $app
    }
}

function Invoke-FvW09 {
    # J-16/J-08: Ctrl+Alt+Shift+L -> the conflicts panel; Ctrl+Alt+Shift+Y
    # -> the needs-you panel; Escape each. The honest quiet states are
    # visible in the captures.
    $app = Start-FlauzApp -SceneId 'fv-w09'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w09' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w09' -Process $app.Process -Keys '^%+l'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w09' -Process $app.Process -Because 'the conflicts-panel drive'
        Capture-FlauzFrame -SceneId 'fv-w09' -Name '01-conflicts-panel-open'
        Send-FlauzKeys -SceneId 'fv-w09' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w09' -Name '02-conflicts-panel-closed'
        Send-FlauzKeys -SceneId 'fv-w09' -Process $app.Process -Keys '^%+y'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w09' -Process $app.Process -Because 'the needs-you-panel drive'
        Capture-FlauzFrame -SceneId 'fv-w09' -Name '03-needs-you-panel-open'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w09' -Name '04-needs-you-honest-quiet'
        Send-FlauzKeys -SceneId 'fv-w09' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w09' -Process $app.Process -Because 'the Escape close'
        Capture-FlauzFrame -SceneId 'fv-w09' -Name '05-needs-you-panel-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w09' -App $app
    }
}

function Invoke-FvW10 {
    # J-07: Ctrl+Alt+Shift+7 -> the agents view; Escape. The honest
    # empty is visible in the capture.
    $app = Start-FlauzApp -SceneId 'fv-w10'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w10' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w10' -Process $app.Process -Keys '^%+7'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w10' -Process $app.Process -Because 'the agents-view drive'
        Capture-FlauzFrame -SceneId 'fv-w10' -Name '01-agents-view-open'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w10' -Name '02-agents-view-honest-empty'
        Send-FlauzKeys -SceneId 'fv-w10' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Assert-FlauzAlive -SceneId 'fv-w10' -Process $app.Process -Because 'the Escape close'
        Capture-FlauzFrame -SceneId 'fv-w10' -Name '03-agents-view-closed'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w10' -App $app
    }
}

function Invoke-FvW11 {
    # DOMAIN-NEUTRAL: type the non-code objective; Enter; visit the
    # model picker. Bounded: no live turn (gap W-5 — named).
    $app = Start-FlauzApp -SceneId 'fv-w11'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w11' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5
        Send-FlauzKeys -SceneId 'fv-w11' -Process $app.Process -Keys '^n'
        Start-Sleep -Seconds 3
        Capture-FlauzFrame -SceneId 'fv-w11' -Name '01-new-task-surface'
        Send-FlauzText -SceneId 'fv-w11' -Process $app.Process -Text 'Research seasonal planting calendars for the community garden'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w11' -Name '02-noncode-objective-typed'
        Send-FlauzKeys -SceneId 'fv-w11' -Process $app.Process -Keys '{ENTER}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w11' -Name '03-after-enter'
        Send-FlauzKeys -SceneId 'fv-w11' -Process $app.Process -Keys '^{ENTER}'
        Start-Sleep -Seconds 5
        Assert-FlauzAlive -SceneId 'fv-w11' -Process $app.Process -Because 'the submit drive'
        Capture-FlauzFrame -SceneId 'fv-w11' -Name '04-task-surface-noncode'
        Send-FlauzKeys -SceneId 'fv-w11' -Process $app.Process -Keys '^%+m'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w11' -Process $app.Process -Because 'the model-picker visit'
        Capture-FlauzFrame -SceneId 'fv-w11' -Name '05-model-picker-domain-neutral'
        Send-FlauzKeys -SceneId 'fv-w11' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Write-SceneLog -SceneId 'fv-w11' -Message 'RUN NOTE (catalog gap W-5): no live turn on the runner (no authenticated runtime) — the honest bounded scene: the objective echo + the domain-neutral surface language'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w11' -App $app
    }
}

function Invoke-FvW12 {
    # A11Y (bounded depth): keyboard-only drill on the entry surface —
    # Tab/Shift+Tab cycles; Ctrl+K palette -> Escape restores; Ctrl+/
    # overlay -> the two-Escape contract; a guarded chord (Ctrl+P
    # no-workspace) answers honestly; a typed probe lands after each
    # close (no swallowed keyboard).
    $app = Start-FlauzApp -SceneId 'fv-w12'
    try {
        Wait-FlauzMainWindow -SceneId 'fv-w12' -Process $app.Process | Out-Null
        Start-Sleep -Seconds 5

        # Tab/Shift+Tab cycles (keyboard focus moves; no crash).
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '{TAB}'
        Start-Sleep -Seconds 1
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '+{TAB}'
        Start-Sleep -Seconds 1
        Assert-FlauzAlive -SceneId 'fv-w12' -Process $app.Process -Because 'the Tab cycle'
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '01-tab-cycle'

        # Ctrl+K palette -> Escape restores; typed probe lands.
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '^k'
        Start-Sleep -Seconds 3
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '02-palette-open'
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '03-palette-escape-restored'
        Send-FlauzText -SceneId 'fv-w12' -Process $app.Process -Text 'a11y probe'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '04-typed-probe-lands'
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '^a'
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '{DEL}'

        # Ctrl+/ overlay -> the two-Escape contract (a query is typed so
        # the first Escape clears it; the second closes).
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '^/'
        Start-Sleep -Seconds 3
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '05-overlay-open'
        Send-FlauzText -SceneId 'fv-w12' -Process $app.Process -Text 'palette'
        Start-Sleep -Seconds 1
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '06-overlay-esc1'
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '07-overlay-esc2-closed'

        # The guarded chord: Ctrl+P with no workspace -> the honest
        # status ("Select a workspace before searching files.").
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '^p'
        Start-Sleep -Seconds 3
        Assert-FlauzAlive -SceneId 'fv-w12' -Process $app.Process -Because 'the guarded-chord drive'
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '08-ctrl-p-honest-status'
        Send-FlauzKeys -SceneId 'fv-w12' -Process $app.Process -Keys '{ESC}'
        Start-Sleep -Seconds 2

        # Final typed probe (no swallowed keyboard after each close).
        Send-FlauzText -SceneId 'fv-w12' -Process $app.Process -Text 'final probe'
        Start-Sleep -Seconds 2
        Capture-FlauzFrame -SceneId 'fv-w12' -Name '09-final-typed-probe'
    }
    finally {
        Stop-FlauzApp -SceneId 'fv-w12' -App $app
    }
}

# ---------------------------------------------------------------------------
# The run
# ---------------------------------------------------------------------------

$allScenes = @{
    'fv-w01' = { Invoke-FvW01 }
    'fv-w02' = { Invoke-FvW02 }
    'fv-w03' = { Invoke-FvW03 }
    'fv-w04' = { Invoke-FvW04 }
    'fv-w05' = { Invoke-FvW05 }
    'fv-w06' = { Invoke-FvW06 }
    'fv-w07' = { Invoke-FvW07 }
    'fv-w08' = { Invoke-FvW08 }
    'fv-w09' = { Invoke-FvW09 }
    'fv-w10' = { Invoke-FvW10 }
    'fv-w11' = { Invoke-FvW11 }
    'fv-w12' = { Invoke-FvW12 }
}

$requested = if ($Scene.Count -gt 0) { $Scene } else { @($allScenes.Keys | Sort-Object) }
foreach ($sceneId in $requested) {
    if (-not $allScenes.ContainsKey($sceneId)) {
        throw "FV-002 windows journey-smoke: unknown scene id '$sceneId' (known: $($allScenes.Keys -join ', '))"
    }
    Invoke-FlauzScene -SceneId $sceneId -Body $allScenes[$sceneId]
}

# The summary + the run lineage record.
$runRecord = [ordered]@{
    v          = 1
    kind       = 'flauz.fv.windows-journey-smoke.run'
    pinned     = '7f660c00407a5741eee975b2274570a200ff576b'
    binary     = $binaryPath
    codex_bin  = if ([string]::IsNullOrWhiteSpace($validCodexBinary)) { $null } else { $validCodexBinary }
    started_at = $script:StartedAtIso
    scenes     = $script:SceneResults
}
$null = New-Item -ItemType Directory -Path $EvidenceRoot -Force
$runRecord.finished_at = [DateTime]::UtcNow.ToString('u')
($runRecord | ConvertTo-Json -Depth 4) | Set-Content -LiteralPath (Join-Path $EvidenceRoot 'RUN.json')

Write-Host ""
Write-Host '=== FV-002 Windows journey-smoke summary ==='
foreach ($result in $script:SceneResults) {
    Write-Host ("  {0} : {1}" -f $result.Scene, $result.Verdict)
}
$failed = @($script:SceneResults | Where-Object { $_.Verdict -ne 'pass' })
if ($failed.Count -gt 0) {
    throw "FV-002 windows journey-smoke: $($failed.Count) scene(s) failed: $($failed.Scene -join ', ')"
}
Write-Host "FV-002 Windows journey-smoke: ALL REQUESTED SCENES PASS — evidence in $EvidenceRoot"
