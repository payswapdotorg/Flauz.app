#!/usr/bin/env bash
set -euo pipefail

# REL-001 — verify a downloaded release archive against the published
# SHA256SUMS manifest (and the detached OpenPGP signature over that
# manifest when one is present). Hard-fails with named messages; never
# fakes a pass and never silently substitutes a weaker check.
#
# Usage:
#   verify_release_archive.sh <archive> <sha256sums-file> [signature-file]
#
# The signature file is optional. When it is not passed, a detached
# signature named <sha256sums-file>.asc or <sha256sums-file>.sig is used
# when it sits next to the manifest. When no signature is present at all,
# the script proceeds with checksum-only verification and says so — the
# named honest bound: releases are unsigned until the operator provisions
# the signing key (see docs/platform-support.md, "Linux distribution
# strategy").
#
# The manifest is verified first (authenticity of the oracle), then the
# archive checksum is checked against the manifest entry (integrity of the
# download). Manifest entry names are compared after stripping any leading
# "./" (the release workflow generates "./<name>" entries).
#
# Exit codes:
#   0  verification passed (checksum — and signature, when present — OK)
#   1  verification FAILED (malformed manifest line, duplicate manifest
#      entries, no manifest entry for the archive, checksum mismatch, or
#      a present signature that does not verify)
#   2  usage / input / environment error (bad arguments, unreadable
#      files, or required tooling — gpg, sha256sum/shasum — absent)

PROG="${0##*/}"

fail() {
  # fail <exit-code> <message...>
  local code="$1"
  shift
  echo "${PROG}: $*" >&2
  exit "${code}"
}

if [[ $# -ne 2 && $# -ne 3 ]]; then
  echo "usage: ${PROG} <archive> <sha256sums-file> [signature-file]" >&2
  exit 2
fi

archive="$1"
manifest="$2"
signature="${3:-}"

[[ -f "${archive}" && -r "${archive}" ]] ||
  fail 2 "VERIFY-INPUT-MISSING: the archive is not a readable file: ${archive}"
[[ -f "${manifest}" && -r "${manifest}" ]] ||
  fail 2 "VERIFY-INPUT-MISSING: the SHA256SUMS file is not a readable file: ${manifest}"

archive_name="${archive##*/}"

if [[ -n "${signature}" ]]; then
  [[ -f "${signature}" && -r "${signature}" ]] ||
    fail 2 "VERIFY-INPUT-MISSING: the signature file is not a readable file: ${signature}"
else
  for candidate in "${manifest}.asc" "${manifest}.sig"; do
    if [[ -f "${candidate}" && -r "${candidate}" ]]; then
      signature="${candidate}"
      break
    fi
  done
fi

# --- detached-signature verification (when one is present) ---------------

if [[ -n "${signature}" ]]; then
  if ! command -v gpg >/dev/null 2>&1; then
    fail 2 "VERIFY-TOOLING-ABSENT: gpg is not installed, so the detached signature ${signature##*/} cannot be verified. Recovery: install gpg (for example 'sudo apt-get install gnupg') and re-run, or verify on a machine that has gpg. A present signature is never skipped silently, so checksum-only verification is NOT substituted here."
  fi
  gpg_status=0
  gpg_output="$(gpg --verify "${signature}" "${manifest}" 2>&1)" || gpg_status=$?
  if [[ "${gpg_status}" -ne 0 ]]; then
    fail 1 "SIGNATURE-VERIFY-FAILED: the detached signature ${signature##*/} does NOT verify against ${manifest##*/} (gpg exit ${gpg_status}): ${gpg_output} — the manifest or the signature was modified, or the signing key's public half is not in your keyring. Do not trust this download."
  fi
  echo "[REL-001] SIGNATURE-VERIFIED: ${signature##*/} is a valid detached signature over ${manifest##*/}"
else
  echo "[REL-001] SIGNATURE-ABSENT: no detached signature found (looked for a third argument and for ${manifest##*/}.asc / .sig next to the manifest) — proceeding with checksum-only verification. Honest bound: current releases are unsigned; the checksum manifest detects corruption after download, it does not authenticate the publisher (see docs/platform-support.md, Linux distribution strategy)."
fi

# --- locate the manifest entry for this archive --------------------------

expected=""
matches=0
line_no=0
while IFS= read -r line || [[ -n "${line}" ]]; do
  line_no=$((line_no + 1))
  line="${line%"${line##*[![:space:]]}"}"   # strip trailing CR/space noise
  [[ -z "${line}" ]] && continue
  if [[ "${line}" =~ ^([0-9A-Fa-f]{64})[[:space:]][[:space:]*]?(.*)$ ]]; then
    entry_hash="${BASH_REMATCH[1]}"
    entry_name="${BASH_REMATCH[2]}"
    entry_name="${entry_name#./}"
    if [[ "${entry_name}" = "${archive_name}" ]]; then
      matches=$((matches + 1))
      expected="$(printf '%s' "${entry_hash}" | tr '[:upper:]' '[:lower:]')"
    fi
  else
    fail 1 "MANIFEST-MALFORMED: ${manifest##*/} line ${line_no} is not a '<sha256>  <name>' entry: ${line}"
  fi
done < "${manifest}"

if [[ "${matches}" -eq 0 ]]; then
  fail 1 "MANIFEST-ENTRY-MISSING: ${manifest##*/} contains no entry for ${archive_name} — the manifest and the archive do not belong to the same release, or the archive was renamed. Do not trust this archive."
fi
if [[ "${matches}" -gt 1 ]]; then
  fail 1 "MANIFEST-DUPLICATE-ENTRIES: ${manifest##*/} contains ${matches} entries for ${archive_name} — the manifest is ambiguous or was tampered with. Do not trust this archive."
fi

# --- checksum verification ------------------------------------------------

if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "${archive}" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  actual="$(shasum -a 256 "${archive}" | awk '{print $1}')"
else
  fail 2 "VERIFY-TOOLING-ABSENT: neither sha256sum nor shasum is installed — cannot compute the archive checksum. Recovery: install coreutils (sha256sum) or a perl-based shasum, then re-run."
fi
actual="$(printf '%s' "${actual}" | tr '[:upper:]' '[:lower:]')"

if [[ "${actual}" != "${expected}" ]]; then
  fail 1 "CHECKSUM-MISMATCH: ${archive_name} does NOT match its SHA256SUMS entry — expected ${expected}, got ${actual}. The archive is corrupted or was modified after the manifest was generated (tamper suspected). Do not extract or run it; re-download from the official GitHub release and re-verify; if it still fails, report it."
fi

echo "[REL-001] CHECKSUM-VERIFIED: ${archive_name} matches its SHA256SUMS entry (${actual})"
if [[ -n "${signature}" ]]; then
  echo "[REL-001] VERIFY-PASSED: ${archive_name} verified against ${manifest##*/} and the detached signature ${signature##*/}"
else
  echo "[REL-001] VERIFY-PASSED: ${archive_name} verified against ${manifest##*/} (checksum-only; no signature was present)"
fi
