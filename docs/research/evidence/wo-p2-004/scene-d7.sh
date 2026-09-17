#!/usr/bin/env bash
# Scene D7 — WO-P2-004: command palette settings-page indexing.
# Contract under proof: every default-nav settings section is reachable from
# the command palette; the palette query "import" (B2 ev/18 gap: "No
# matches") now resolves and navigates to the Import settings page, and the
# other five previously-missing pages (Profile, Browser, Configuration,
# Hooks, Git) resolve the same way. Unauthenticated fresh workspace.
source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
CAPTURE d7-001-boot

# Palette top (empty query): the Settings group now lists all indexed pages.
XDO key ctrl+k; sleep 1.2
CAPTURE d7-002-palette-top-settings-group
XDO key Escape; sleep 0.6

# The ev/18 remediation: query "import" resolves (was: "No matches").
XDO key ctrl+k; sleep 1.0
XDO type --delay 60 "import"; sleep 1.0
CAPTURE d7-003-palette-import-resolves
XDO key Return; sleep 2.0
CAPTURE d7-004-import-settings-page

# Remaining previously-missing pages, each via palette query + navigation.
XDO key ctrl+k; sleep 1.0
XDO type --delay 60 "profile"; sleep 1.0
CAPTURE d7-005-palette-profile-resolves
XDO key Return; sleep 2.0
CAPTURE d7-006-profile-settings-page

XDO key ctrl+k; sleep 1.0
XDO type --delay 60 "browser"; sleep 1.0
CAPTURE d7-007-palette-browser-resolves
XDO key Return; sleep 2.0
CAPTURE d7-008-browser-settings-page

XDO key ctrl+k; sleep 1.0
XDO type --delay 60 "configuration"; sleep 1.0
CAPTURE d7-009-palette-configuration-resolves
XDO key Return; sleep 2.0
CAPTURE d7-010-configuration-settings-page

XDO key ctrl+k; sleep 1.0
XDO type --delay 60 "hooks"; sleep 1.0
CAPTURE d7-011-palette-hooks-resolves
XDO key Return; sleep 2.0
CAPTURE d7-012-hooks-settings-page

XDO key ctrl+k; sleep 1.0
XDO type --delay 60 "git"; sleep 1.0
CAPTURE d7-013-palette-git-resolves
XDO key Return; sleep 2.0
CAPTURE d7-014-git-settings-page
