#!/usr/bin/env python3
"""Isolated Linux experiment; no product or normal-CI scheduling changes."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

profile, output, *command = sys.argv[1:]
if profile not in ("available", "one-cpu", "contended") or not command:
    raise SystemExit("usage: run.py available|one-cpu|contended OUTPUT COMMAND...")
output = Path(output)
output.mkdir(parents=True, exist_ok=True)
allowed = sorted(os.sched_getaffinity(0))
selected = allowed if profile == "available" else allowed[:1]
os.sched_setaffinity(0, selected)
started = time.time()
metadata = {"profile": profile, "allowedCpus": allowed, "selectedCpus": selected,
            "competitors": 1 if profile == "contended" else 0,
            "command": command, "startedAtEpoch": started}
(output / "scheduling.json").write_text(json.dumps(metadata, indent=2) + "\n")
competitor = None
child = None
try:
    if profile == "contended":
        competitor = subprocess.Popen([sys.executable, "-c", "while True: pass"], start_new_session=True)
        metadata["competitorPid"] = competitor.pid
    child = subprocess.Popen(command, start_new_session=True)
    metadata["commandPid"] = child.pid
    with (output / "host-samples.jsonl").open("w", buffering=1) as samples:
        while True:
            sample = {"elapsedSeconds": time.time() - started,
                      "loadAverage": os.getloadavg(),
                      "cpuTicks": [line for line in Path("/proc/stat").read_text().splitlines()
                                   if line.split()[0] in {"cpu" + str(cpu) for cpu in selected}],
                      "commandExit": child.poll()}
            pressure = Path("/proc/pressure/cpu")
            if pressure.exists():
                sample["hostCpuPressure"] = pressure.read_text()
            samples.write(json.dumps(sample) + "\n")
            if sample["commandExit"] is not None:
                break
            time.sleep(2)
    metadata["exitCode"] = child.returncode
finally:
    for process in (child, competitor):
        if process is not None:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
    metadata["elapsedSeconds"] = time.time() - started
    (output / "scheduling.json").write_text(json.dumps(metadata, indent=2) + "\n")
raise SystemExit(metadata.get("exitCode", 1))
