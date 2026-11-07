#!/usr/bin/env python3
"""One-command runner for the local evaluation harnesses.

Lives in scripts/ (committed); the meetings, transcripts and drafts it produces
stay under eval/<label>/, which is git-ignored except README.md and .gitignore,
so nothing a run touches is ever committed. Nothing is ever sent anywhere except
to the provider endpoint you configure (default: loopback).

Meeting directory shape (see README.md):

    eval/<label>/
    ├── audio.<ext>        the recording (needed for `transcribe`)
    ├── reference.md       the protocol a person wrote (used by `report`)
    ├── notes.md           why the example is useful
    ├── .workspace/        kept by `transcribe`, reused by later stages
    └── runs/<timestamp>/  logs and drafts, one folder per run

Configuration precedence: CLI flags > environment > eval/config.env > defaults.
config.env holds KEY=VALUE lines, e.g.:

    LOCALOG_PROVIDER_ENDPOINT=http://lab-box:11434
    LOCALOG_E2E_WHISPER=/opt/sidecars/whisper-cli
    LOCALOG_E2E_WHISPER_MODEL=/models/ggml-medium.bin
    LOCALOG_E2E_SEG_MODEL=/models/segmentation.onnx
    LOCALOG_E2E_EMB_MODEL=/models/embedding.onnx
    LOCALOG_E2E_DIARISER=/opt/sidecars/sherpa-onnx-offline-speaker-diarization
    LOCALOG_EVAL_LANGUAGE=German
    LOCALOG_EVAL_CONTEXT=40960
    LOCALOG_EVAL_OUTPUT=8192
    LOCALOG_ADHERENCE_SEEDS=7,8,9
    LOCALOG_E2E_VOCABULARY=Term one=project;Term two=person

Commands:

    check        verify cargo, the endpoint, installed models, sidecar paths
    transcribe   full e2e pipeline on the meeting's audio (keeps .workspace)
    sweep        model x seed adherence table from the kept transcript
    generate     one protocol from the kept workspace with one model
    topics       the topic pass alone, from the kept workspace
    report       aggregate the newest run into summary.md + mechanical figures

Every command accepts --dry-run to print the exact cargo invocation and
environment without running anything.
"""

import argparse
import json
import os
import re
import sqlite3
import subprocess
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SRC_TAURI = REPO / "src-tauri"
EVAL = REPO / "eval"
CONFIG_ENV = EVAL / "config.env"

AUDIO_EXT = {".mp3", ".wav", ".m4a", ".mp4", ".mov", ".mkv", ".webm", ".ogg", ".flac"}

DEFAULTS = {
    "LOCALOG_PROVIDER_ENDPOINT": "http://127.0.0.1:11434",
    "LOCALOG_EVAL_LANGUAGE": "German",
    "LOCALOG_EVAL_CONTEXT": "40960",
    "LOCALOG_EVAL_OUTPUT": "8192",
    "LOCALOG_ADHERENCE_SEEDS": "7,8,9",
    "LOCALOG_E2E_LANGUAGE": "German",
    "LOCALOG_E2E_SPEAKERS": "auto",
    "LOCALOG_E2E_MODEL": "qwen3.5:4b",
}


def load_config_env():
    values = {}
    if CONFIG_ENV.exists():
        for line in CONFIG_ENV.read_text().splitlines():
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, _, value = line.partition("=")
            values[key.strip()] = value.strip()
    return values


class Config:
    """CLI flag > process environment > eval/config.env > built-in default."""

    def __init__(self, args):
        file_values = load_config_env()
        self.values = dict(DEFAULTS)
        self.values.update(file_values)
        for key in list(self.values):
            env_value = os.environ.get(key)
            if env_value is not None:
                self.values[key] = env_value
        if getattr(args, "endpoint", None):
            self.values["LOCALOG_PROVIDER_ENDPOINT"] = args.endpoint

    def get(self, key, default=""):
        return self.values.get(key, default)


def find_audio(meeting: Path):
    candidates = [p for p in sorted(meeting.iterdir()) if p.suffix.lower() in AUDIO_EXT]
    if not candidates:
        sys.exit(f"no audio file in {meeting} (expected one of: {', '.join(sorted(AUDIO_EXT))})")
    return candidates[0]


def workspace_db(meeting: Path):
    db = meeting / ".workspace" / "localog.sqlite3"
    if not db.exists():
        sys.exit(f"no kept workspace at {db} — run `transcribe` first")
    return db


def latest_transcript_artifact(meeting: Path):
    """The newest committed transcript JSON in the kept workspace."""
    db = workspace_db(meeting)
    con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    try:
        row = con.execute(
            "SELECT artifact_path FROM transcript_revisions ORDER BY created_at_ms DESC LIMIT 1"
        ).fetchone()
    finally:
        con.close()
    if not row:
        sys.exit(f"no transcript revision in {db}")
    # artifact_path is stored relative to the workspace root; accept an absolute
    # path too, but never one that escapes the meeting directory.
    root = meeting / ".workspace"
    for candidate in (root / row[0], meeting / row[0], Path(row[0])):
        if candidate.exists() and (candidate.is_absolute() and meeting in candidate.parents
                                    or not candidate.is_absolute()):
            return candidate.resolve()
    sys.exit(f"transcript artifact missing: {root / row[0]}")


def new_run_dir(meeting: Path, dry_run=False):
    stamp = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")
    run = meeting / "runs" / stamp
    if not dry_run:
        run.mkdir(parents=True, exist_ok=True)
    return run


def cargo_test(cfg, test_name, env_extra, run_dir, log_name, dry_run):
    env = dict(os.environ)
    env.update({k: v for k, v in cfg.values.items() if v})
    env["LOCALOG_REAL_RUNTIMES"] = "1"
    env.update(env_extra)
    cmd = ["cargo", "test", "--lib", "--", "--ignored", "--nocapture", test_name]
    print(f"$ (cwd={SRC_TAURI}) {' '.join(cmd)}")
    for key in sorted(env_extra):
        shown = env_extra[key]
        print(f"    {key}={shown}")
    if dry_run:
        print("  (dry run — nothing executed)")
        return None
    log = run_dir / log_name
    with log.open("w") as handle:
        handle.write(f"$ {' '.join(cmd)}\n")
        for key in sorted(env_extra):
            handle.write(f"{key}={env_extra[key]}\n")
        handle.flush()
        proc = subprocess.run(cmd, cwd=SRC_TAURI, env=env, stdout=handle, stderr=subprocess.STDOUT)
    print(f"  exit={proc.returncode} log={log}")
    if proc.returncode != 0:
        print(f"  harness failed — see {log}")
    return proc.returncode


def cmd_check(cfg, args):
    print(f"repo:        {REPO}")
    print(f"cargo:       ", end="")
    try:
        print(subprocess.run(["cargo", "--version"], capture_output=True, text=True).stdout.strip())
    except FileNotFoundError:
        print("MISSING")
    endpoint = cfg.get("LOCALOG_PROVIDER_ENDPOINT")
    print(f"endpoint:    {endpoint}")
    # Ollama flavor answers /api/version + /api/tags; OpenAI- and
    # Anthropic-compatible servers (LM Studio, llama-server, vLLM, ...)
    # list models at /v1/models instead.
    try:
        with urllib.request.urlopen(f"{endpoint}/api/version", timeout=5) as response:
            version = json.load(response).get("version")
        if not version:
            raise ValueError("not an Ollama /api/version body")
        print(f"  format:    ollama {version}")
        request = urllib.request.Request(
            f"{endpoint}/api/tags", data=b"{}", headers={"Content-Type": "application/json"}
        )
        with urllib.request.urlopen(request, timeout=5) as response:
            models = [m["name"] for m in json.load(response).get("models", [])]
    except Exception:
        try:
            with urllib.request.urlopen(f"{endpoint}/v1/models", timeout=5) as response:
                models = [m["id"] for m in json.load(response).get("data", [])]
            print("  format:    openai/anthropic-compatible")
        except Exception as error:  # noqa: BLE001 - a check reports, it does not crash
            print(f"  UNREACHABLE: {error}")
            models = []
    print(f"  models:    {', '.join(models) if models else '(none listed)'}")
    for key in ("LOCALOG_E2E_WHISPER", "LOCALOG_E2E_DIARISER", "LOCALOG_E2E_WHISPER_MODEL",
                "LOCALOG_E2E_SEG_MODEL", "LOCALOG_E2E_EMB_MODEL"):
        value = cfg.get(key)
        status = "ok" if value and Path(value).exists() else ("MISSING" if not value else "NOT FOUND")
        print(f"{key}: {value or '(unset)'} [{status}]")


def cmd_transcribe(cfg, args):
    meeting = Path(args.meeting).resolve()
    audio = find_audio(meeting)
    run = new_run_dir(meeting, args.dry_run)
    env = {
        "LOCALOG_E2E_AUDIO": str(audio),
        "LOCALOG_E2E_ROOT": str(meeting / ".workspace"),
        "LOCALOG_E2E_OUT": str(run / f"protocol-{cfg.get('LOCALOG_E2E_MODEL')}.md"),
        "LOCALOG_E2E_LANGUAGE": cfg.get("LOCALOG_E2E_LANGUAGE"),
        "LOCALOG_E2E_SPEAKERS": cfg.get("LOCALOG_E2E_SPEAKERS"),
        "LOCALOG_E2E_MODEL": args.model or cfg.get("LOCALOG_E2E_MODEL"),
    }
    for key in ("LOCALOG_E2E_WHISPER", "LOCALOG_E2E_DIARISER", "LOCALOG_E2E_WHISPER_MODEL",
                "LOCALOG_E2E_SEG_MODEL", "LOCALOG_E2E_EMB_MODEL", "LOCALOG_E2E_VOCABULARY"):
        if cfg.get(key):
            env[key] = cfg.get(key)
    return cargo_test(cfg, "runs_the_whole_pipeline_on_a_real_meeting", env, run, "transcribe.log",
                      args.dry_run)


def cmd_sweep(cfg, args):
    meeting = Path(args.meeting).resolve()
    transcript = Path(args.transcript).resolve() if args.transcript else latest_transcript_artifact(meeting)
    run = new_run_dir(meeting, args.dry_run)
    out = run / "adherence"
    if not args.dry_run:
        out.mkdir(exist_ok=True)
    models = args.models or cfg.get("LOCALOG_ADHERENCE_MODELS", "qwen3.5:4b")
    env = {
        "LOCALOG_ADHERENCE_TRANSCRIPT": str(transcript),
        "LOCALOG_ADHERENCE_OUT": str(out),
        "LOCALOG_ADHERENCE_MODELS": models,
        "LOCALOG_ADHERENCE_SEEDS": args.seeds or cfg.get("LOCALOG_ADHERENCE_SEEDS"),
        "LOCALOG_EVAL_LANGUAGE": cfg.get("LOCALOG_EVAL_LANGUAGE"),
        "LOCALOG_EVAL_CONTEXT": cfg.get("LOCALOG_EVAL_CONTEXT"),
        "LOCALOG_EVAL_OUTPUT": cfg.get("LOCALOG_EVAL_OUTPUT"),
        "LOCALOG_PROVIDER_ENDPOINT": cfg.get("LOCALOG_PROVIDER_ENDPOINT"),
    }
    print(f"transcript:  {transcript}")
    print(f"models:      {models}  seeds: {env['LOCALOG_ADHERENCE_SEEDS']}")
    return cargo_test(cfg, "does_the_model_follow_the_style", env, run, "sweep.log", args.dry_run)


def cmd_generate(cfg, args):
    meeting = Path(args.meeting).resolve()
    workspace_db(meeting)  # fail early with a clear message
    run = new_run_dir(meeting, args.dry_run)
    model = args.model or cfg.get("LOCALOG_E2E_MODEL")
    env = {
        "LOCALOG_E2E_ROOT": str(meeting / ".workspace"),
        "LOCALOG_E2E_MODEL": model,
        "LOCALOG_E2E_OUT": str(run / f"protocol-{model.replace(':', '-')}.md"),
    }
    return cargo_test(cfg, "generates_from_an_existing_workspace", env, run, "generate.log",
                      args.dry_run)


def cmd_topics(cfg, args):
    meeting = Path(args.meeting).resolve()
    workspace_db(meeting)
    run = new_run_dir(meeting, args.dry_run)
    env = {
        "LOCALOG_E2E_ROOT": str(meeting / ".workspace"),
        "LOCALOG_E2E_MODEL": args.model or cfg.get("LOCALOG_E2E_MODEL"),
        "LOCALOG_PROVIDER_ENDPOINT": cfg.get("LOCALOG_PROVIDER_ENDPOINT"),
    }
    return cargo_test(cfg, "finds_the_topics_of_a_real_meeting", env, run, "topics.log", args.dry_run)


# Numbers next to a unit worth recognising; deliberately short, like facts.rs.
UNITS = [
    "Quadratmeter", "qm", "m²", "Zentimeter", "Millimeter", "Kilometer", "Meter",
    "Prozent", "%", "Euro", "€", "Mio", "Mrd", "Tsd", "Stück",
]
QUANTITY = re.compile(
    r"(\d[\d.,]*)\s?(" + "|".join(re.escape(u) for u in UNITS) + r")\b", re.IGNORECASE
)


def quantities(text):
    return {m.group(0).lower() for m in QUANTITY.finditer(text)}


def cmd_report(cfg, args):
    meeting = Path(args.meeting).resolve()
    runs = sorted((meeting / "runs").glob("*")) if (meeting / "runs").exists() else []
    if not runs:
        sys.exit(f"no runs under {meeting / 'runs'} — run transcribe/sweep/generate first")
    run = runs[-1]
    reference = meeting / "reference.md"
    lines = [f"# Evaluation summary — {meeting.name} — run {run.name}", ""]
    if reference.exists():
        ref_q = quantities(reference.read_text())
        lines.append(f"reference.md: {reference.stat().st_size} bytes, {len(ref_q)} quantities found")
    else:
        ref_q = None
        lines.append("reference.md: (absent — figure comparison skipped)")
    lines.append("")
    for adherence in sorted(run.glob("adherence/adherence.md")):
        lines += [f"## Adherence table ({adherence.parent.name})", "", "```", adherence.read_text(), "```", ""]
    protocols = sorted(run.glob("adherence/protocol-*.md")) + sorted(run.glob("protocol-*.md"))
    if protocols:
        lines += ["## Drafts", "", "| file | chars | quantities kept (vs reference) |", "| --- | ---: | ---: |"]
        for protocol in protocols:
            text = protocol.read_text()
            kept = ""
            if ref_q:
                kept = f"{len(quantities(text) & ref_q)}/{len(ref_q)}"
            lines.append(f"| {protocol.name} | {len(text)} | {kept or '—'} |")
        lines.append("")
    for log in sorted(run.glob("*.log")):
        lines += [f"## Log: {log.name}", "", "```", log.read_text()[-4000:], "```", ""]
    summary = run / "summary.md"
    summary.write_text("\n".join(lines) + "\n")
    print(f"wrote {summary}")
    print("\n".join(lines[:40]))


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--endpoint", help="Ollama-compatible endpoint (default http://127.0.0.1:11434)")
    parser.add_argument("--dry-run", action="store_true", help="print commands and env, run nothing")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("check", help="verify prerequisites").set_defaults(func=cmd_check)

    p = sub.add_parser("transcribe", help="full e2e pipeline on the meeting's audio")
    p.add_argument("meeting", help="eval/<label>/ directory")
    p.add_argument("--model", help="generation model for the pipeline run")
    p.set_defaults(func=cmd_transcribe)

    p = sub.add_parser("sweep", help="model x seed adherence sweep from the kept transcript")
    p.add_argument("meeting")
    p.add_argument("--models", help="comma-separated model names")
    p.add_argument("--seeds", help="comma-separated seeds (default 7,8,9)")
    p.add_argument("--transcript", help="transcript JSON (default: newest in .workspace)")
    p.set_defaults(func=cmd_sweep)

    p = sub.add_parser("generate", help="one protocol from the kept workspace")
    p.add_argument("meeting")
    p.add_argument("--model")
    p.set_defaults(func=cmd_generate)

    p = sub.add_parser("topics", help="topic pass alone from the kept workspace")
    p.add_argument("meeting")
    p.add_argument("--model")
    p.set_defaults(func=cmd_topics)

    p = sub.add_parser("report", help="aggregate the newest run into summary.md")
    p.add_argument("meeting")
    p.set_defaults(func=cmd_report)

    args = parser.parse_args()
    cfg = Config(args)
    if args.command == "check":
        cmd_check(cfg, args)
        return
    sys.exit(args.func(cfg, args) or 0)


if __name__ == "__main__":
    main()
