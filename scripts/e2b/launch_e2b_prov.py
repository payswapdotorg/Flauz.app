#!/usr/bin/env python3
"""launch_e2b_prov.py — detached launcher for the E2B provision/build.

Prints the child pid and exits IMMEDIATELY so the child reparents to init
before the invoking tool shell returns (boot-prompt lesson 13/62: children
started with plain nohup/setsid from a tool shell are reaped when the
invocation ends).
"""
import subprocess
import sys

LOG = sys.argv[1] if len(sys.argv) > 1 else "/home/z/flauz-lane/e2b/prov.log"
CMD = sys.argv[2:]

with open(LOG, "a") as lf:
    p = subprocess.Popen(
        ["/home/z/.venv/bin/python3"] + CMD,
        stdout=lf, stderr=subprocess.STDOUT,
        start_new_session=True, cwd="/home/z/flauz-lane/e2b")
print(p.pid)
