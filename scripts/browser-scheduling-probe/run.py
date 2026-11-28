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
def browser_processes():
    records = []
    for entry in Path('/proc').iterdir():
        if not entry.name.isdigit():
            continue
        try:
            name = (entry / 'comm').read_text().strip()
            executable = Path(os.readlink(entry / 'exe')).name
            if not any(token in (name + ' ' + executable).lower()
                       for token in ('webkit', 'minibrowser', 'wpe', 'webprocess', 'networkprocess', 'gpuprocess')):
                continue
            # Deliberately omit arguments and environments. stat fields are
            # numeric counters, state and parent ID for repository browser runs.
            stat = (entry / 'stat').read_text().rsplit(')', 1)[1].split()
            records.append({'pid': int(entry.name), 'name': name, 'executable': executable, 'state': stat[0],
                            'ppid': int(stat[1]), 'userTicks': int(stat[11]),
                            'systemTicks': int(stat[12]), 'threads': int(stat[17]),
                            'startedTicks': int(stat[19]),
                            'waitChannel': (entry / 'wchan').read_text().strip()})
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            pass
    return records

started = time.time()
metadata = {"profile": profile, "allowedCpus": allowed, "selectedCpus": selected,
            "competitors": 1 if profile == "contended" else 0,
            "command": command, "startedAtEpoch": started,
            "clockTicksPerSecond": os.sysconf('SC_CLK_TCK'),
            "cpuModel": next((line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name')), None)}
(output / "scheduling.json").write_text(json.dumps(metadata, indent=2) + "\n")
competitor = None
child = None
try:
    if profile == "contended":
        competitor = subprocess.Popen([sys.executable, "-c", "while True: pass"], start_new_session=True)
        metadata["competitorPid"] = competitor.pid
    child = subprocess.Popen(command, start_new_session=True)
    metadata["commandPid"] = child.pid
    process_names = set()
    with (output / "host-samples.jsonl").open("w", buffering=1) as samples:
        while True:
            sample = {"elapsedSeconds": time.time() - started,
                      "loadAverage": os.getloadavg(),
                      "cpuTicks": [line for line in Path("/proc/stat").read_text().splitlines()
                                   if line.split()[0] in {"cpu" + str(cpu) for cpu in selected}],
                      "commandExit": child.poll(), "browserProcesses": browser_processes()}
            process_names.update(p['executable'] for p in sample['browserProcesses'])
            pressure = Path("/proc/pressure/cpu")
            if pressure.exists():
                sample["hostCpuPressure"] = pressure.read_text()
            samples.write(json.dumps(sample) + "\n")
            if sample["commandExit"] is not None:
                break
            time.sleep(2)
    metadata["exitCode"] = child.returncode
    metadata["capturedExecutables"] = sorted(process_names)
    metadata["capturedWebProcess"] = any('webprocess' in name.lower() for name in process_names)
    if not metadata["capturedWebProcess"]:
        raise RuntimeError("Native capture lacks a rendering web process; do not infer idle renderer state")
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
