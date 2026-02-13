#!/usr/bin/env python3.11
"""
BMAD Prompt-to-Product Pipeline (Phase 3: Building)

Autonomous Dev/QA pair programming loop for Zellij Session Intelligence plugin.
Uses Anthropic API directly with tool use -- no subprocess spawning.

Usage:
    python3.11 bmad_prompt2product.py                     # Build all stories
    python3.11 bmad_prompt2product.py --story 1.1          # Single story
    python3.11 bmad_prompt2product.py --start-from 2.1     # Resume from story
    python3.11 bmad_prompt2product.py --dry-run             # Preview only
    python3.11 bmad_prompt2product.py --builder sonnet      # Cheaper dev model

Requires: pip install anthropic
Requires: ANTHROPIC_API_KEY environment variable
"""

import subprocess
import sys
import argparse
import json
import os
import glob as glob_module
from pathlib import Path
from datetime import datetime

import anthropic


# ──────────────────────────────────────────────────────────────
# Project Paths
# ──────────────────────────────────────────────────────────────
PROJECT_ROOT = Path(__file__).parent.resolve()
STORIES_DIR = PROJECT_ROOT / "_bmad-output" / "stories"
PLANNING_DIR = PROJECT_ROOT / "_bmad-output" / "planning-artifacts"
LOG_DIR = PROJECT_ROOT / "_bmad-output" / "pipeline-logs"
ARCHITECTURE_DOC = PLANNING_DIR / "architecture.md"


# ──────────────────────────────────────────────────────────────
# Terminal Colors
# ──────────────────────────────────────────────────────────────
class C:
    DEV  = "\033[96m"
    QA   = "\033[93m"
    SYS  = "\033[90m"
    TOOL = "\033[35m"
    PASS = "\033[92m"
    FAIL = "\033[91m"
    WARN = "\033[93m"
    BOLD = "\033[1m"
    DIM  = "\033[2m"
    R    = "\033[0m"


# ──────────────────────────────────────────────────────────────
# Model Registry
# ──────────────────────────────────────────────────────────────
MODEL_MAP = {
    "opus":   "claude-opus-4-6",
    "sonnet": "claude-sonnet-4-5-20250929",
    "haiku":  "claude-haiku-4-5-20251001",
}


def resolve_model(alias: str) -> str:
    return MODEL_MAP.get(alias, alias)


# ──────────────────────────────────────────────────────────────
# Tool Definitions
# ──────────────────────────────────────────────────────────────
TOOLS = [
    {
        "name": "read_file",
        "description": "Read the contents of a file. Returns the file content as a string.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute or project-relative file path to read"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "write_file",
        "description": "Write content to a file. Creates the file and any parent directories if they don't exist. Overwrites existing content.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute or project-relative file path to write"
                },
                "content": {
                    "type": "string",
                    "description": "The full content to write to the file"
                }
            },
            "required": ["path", "content"]
        }
    },
    {
        "name": "list_directory",
        "description": "List files in a directory, optionally matching a glob pattern. Returns file paths one per line.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list"
                },
                "pattern": {
                    "type": "string",
                    "description": "Optional glob pattern (e.g. '*.rs', '**/*.toml'). Default: '*'"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "search_files",
        "description": "Search file contents for a pattern (like grep). Returns matching lines with file paths and line numbers.",
        "input_schema": {
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Text or regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file to search in (default: project root)"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "Glob to filter which files to search (e.g. '*.rs')"
                }
            },
            "required": ["pattern"]
        }
    },
    {
        "name": "bash",
        "description": "Execute a shell command and return stdout + stderr. Use for cargo build, cargo check, git, etc.",
        "input_schema": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        }
    },
]


# ──────────────────────────────────────────────────────────────
# Tool Execution
# ──────────────────────────────────────────────────────────────
def resolve_path(path_str: str) -> Path:
    """Resolve a path relative to project root if not absolute."""
    p = Path(path_str)
    if p.is_absolute():
        return p
    return PROJECT_ROOT / p


def execute_tool(name: str, input_data: dict) -> str:
    """Execute a tool and return the result as a string."""
    try:
        if name == "read_file":
            path = resolve_path(input_data["path"])
            if not path.exists():
                return f"Error: File not found: {path}"
            content = path.read_text(errors="replace")
            # Truncate very large files
            if len(content) > 100_000:
                content = content[:100_000] + f"\n\n... [truncated, {len(content)} chars total]"
            return content

        elif name == "write_file":
            path = resolve_path(input_data["path"])
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(input_data["content"])
            return f"File written: {path} ({len(input_data['content'])} chars)"

        elif name == "list_directory":
            path = resolve_path(input_data["path"])
            pattern = input_data.get("pattern", "*")
            if not path.exists():
                return f"Error: Directory not found: {path}"
            files = sorted(path.glob(pattern))
            if not files:
                return f"No files matching '{pattern}' in {path}"
            return "\n".join(str(f.relative_to(PROJECT_ROOT)) for f in files[:200])

        elif name == "search_files":
            search_path = resolve_path(input_data.get("path", "."))
            pattern = input_data["pattern"]
            file_pattern = input_data.get("file_pattern", "")
            cmd = ["grep", "-rn", "--include", file_pattern, pattern, str(search_path)] if file_pattern else ["grep", "-rn", pattern, str(search_path)]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            output = result.stdout[:50_000]
            if not output:
                return f"No matches for '{pattern}'"
            return output

        elif name == "bash":
            command = input_data["command"]
            result = subprocess.run(
                command, shell=True, capture_output=True, text=True,
                timeout=120, cwd=str(PROJECT_ROOT)
            )
            output = ""
            if result.stdout:
                output += result.stdout
            if result.stderr:
                output += f"\n[stderr]\n{result.stderr}"
            if result.returncode != 0:
                output += f"\n[exit code: {result.returncode}]"
            # Truncate
            if len(output) > 50_000:
                output = output[:50_000] + "\n... [truncated]"
            return output or "(no output)"

        else:
            return f"Error: Unknown tool '{name}'"

    except subprocess.TimeoutExpired:
        return "Error: Command timed out (120s limit)"
    except Exception as e:
        return f"Error: {type(e).__name__}: {e}"


# ──────────────────────────────────────────────────────────────
# Display Helpers
# ──────────────────────────────────────────────────────────────
def phase_banner(title: str, description: str):
    print(f"\n{C.BOLD}{C.SYS}")
    print(f"{'=' * 70}")
    print(f"  {title}")
    print(f"  {description}")
    print(f"  Mode: AUTONOMOUS - just watch")
    print(f"{'=' * 70}")
    print(f"{C.R}\n")


def agent_banner(agent: str, model: str, action: str):
    color = C.DEV if agent == "dev" else C.QA
    icon = "\U0001F4BB" if agent == "dev" else "\U0001F9EA"
    name = "Amelia (Dev)" if agent == "dev" else "Quinn (QA)"
    print(f"\n{color}{C.BOLD}{'─' * 60}")
    print(f" {icon} {name} | model: {model}")
    print(f" {action}")
    print(f"{'─' * 60}{C.R}\n")


def gate_banner(status: str):
    colors = {"PASS": C.PASS, "FAIL": C.FAIL, "CONCERNS": C.WARN}
    color = colors.get(status, C.SYS)
    print(f"\n{color}{C.BOLD}>>> GATE: {status} <<<{C.R}\n")


def tool_banner(name: str, input_data: dict):
    """Show what tool is being called."""
    summary = ""
    if name == "read_file":
        summary = input_data.get("path", "")
    elif name == "write_file":
        summary = input_data.get("path", "")
    elif name == "bash":
        cmd = input_data.get("command", "")
        summary = cmd[:80] + ("..." if len(cmd) > 80 else "")
    elif name == "list_directory":
        summary = f"{input_data.get('path', '')} {input_data.get('pattern', '*')}"
    elif name == "search_files":
        summary = f"'{input_data.get('pattern', '')}'"
    print(f"{C.TOOL}{C.DIM}  [{name}] {summary}{C.R}")


# ──────────────────────────────────────────────────────────────
# System Prompts
# ──────────────────────────────────────────────────────────────
DEV_SYSTEM_PROMPT = f"""You are Amelia, a Developer Agent building the Zellij Session Intelligence WASM plugin.
You are an expert Rust developer experienced with WebAssembly and terminal multiplexers.

PROJECT:
- Zellij WASM plugin at default-plugins/session-intelligence/
- AI-powered session memory sidebar for the Zellij terminal multiplexer
- Uses zellij-tile crate, compiled to wasm32-wasip1
- Project root: {PROJECT_ROOT}

WORKFLOW:
1. Read the story file to understand what to implement
2. Read the architecture doc at {ARCHITECTURE_DOC} for technical decisions
3. Explore existing plugins in default-plugins/ for patterns (e.g., strider, status-bar)
4. Implement each task in order as listed in the story
5. After implementing, run: cargo check --manifest-path default-plugins/session-intelligence/Cargo.toml
6. Fix any compilation errors before finishing

RULES:
- Implement tasks IN ORDER as listed in the story
- Never panic -- all errors handled with match/if-let + fallback
- Use eprintln! for logging
- Follow Rust naming: snake_case functions/fields, PascalCase types
- Route all state through PluginState -- no side-channel state
- Pick the best approach and explain rationale -- do NOT ask for confirmation
- When done, summarize what was implemented and any decisions made"""

QA_SYSTEM_PROMPT = f"""You are Quinn, a QA Engineer reviewing code for the Zellij Session Intelligence WASM plugin.
You are the human's proxy reviewer. Be constructive but thorough.

PROJECT:
- Zellij WASM plugin at default-plugins/session-intelligence/
- Rust code compiled to wasm32-wasip1 using zellij-tile crate
- Architecture doc: {ARCHITECTURE_DOC}
- Project root: {PROJECT_ROOT}

REVIEW PROCESS:
1. Read all source files in default-plugins/session-intelligence/src/
2. Read the story to understand acceptance criteria
3. Check each acceptance criterion -- is it met?
4. Run: cargo check --manifest-path default-plugins/session-intelligence/Cargo.toml
5. Look for: missing error handling, panics, security issues, over-engineering

OUTPUT FORMAT (you MUST end your review with this):
GATE: [PASS|CONCERNS|FAIL]
REQUIRED_FIXES: [numbered list of specific issues, or 'none']
NOTES: [optional observations]"""


# ──────────────────────────────────────────────────────────────
# Agentic Loop
# ──────────────────────────────────────────────────────────────
class Agent:
    def __init__(self, agent_id: str, model_alias: str, client: anthropic.Anthropic):
        self.id = agent_id
        self.model_alias = model_alias
        self.model = resolve_model(model_alias)
        self.client = client
        self.system_prompt = DEV_SYSTEM_PROMPT if agent_id == "dev" else QA_SYSTEM_PROMPT

    def run(self, prompt: str, context: str = "") -> str:
        """Run agentic tool-use loop until the model is done."""
        color = C.DEV if self.id == "dev" else C.QA

        # Build initial message
        user_content = prompt
        if context:
            user_content = f"Context from previous step:\n---\n{context}\n---\n\n{prompt}"

        messages = [{"role": "user", "content": user_content}]
        all_text = []
        turn = 0
        max_turns = 50  # safety limit

        while turn < max_turns:
            turn += 1

            response = self.client.messages.create(
                model=self.model,
                max_tokens=16384,
                system=self.system_prompt,
                tools=TOOLS,
                messages=messages,
            )

            # Process response content blocks
            tool_uses = []
            for block in response.content:
                if block.type == "text":
                    print(f"{color}{block.text}{C.R}")
                    all_text.append(block.text)
                elif block.type == "tool_use":
                    tool_uses.append(block)
                    tool_banner(block.name, block.input)

            # If no tool use, we're done
            if response.stop_reason == "end_turn" or not tool_uses:
                break

            # Execute tools and build tool results
            messages.append({"role": "assistant", "content": response.content})

            tool_results = []
            for tool_use in tool_uses:
                result = execute_tool(tool_use.name, tool_use.input)
                # Show abbreviated result for bash/search
                if tool_use.name in ("bash", "search_files"):
                    preview = result[:200].replace("\n", " ")
                    if len(result) > 200:
                        preview += "..."
                    print(f"{C.TOOL}{C.DIM}  -> {preview}{C.R}")
                tool_results.append({
                    "type": "tool_result",
                    "tool_use_id": tool_use.id,
                    "content": result,
                })

            messages.append({"role": "user", "content": tool_results})

        output = "\n".join(all_text)
        self._log(prompt, output, turn)
        return output

    def _log(self, prompt: str, output: str, turns: int):
        LOG_DIR.mkdir(parents=True, exist_ok=True)
        ts = datetime.now().strftime("%Y%m%d-%H%M%S")
        log_file = LOG_DIR / f"{self.id}-{ts}.md"
        log_file.write_text(
            f"# {self.id} ({self.model}) - {turns} turns\n\n"
            f"## Prompt\n\n{prompt[:3000]}\n\n"
            f"## Output\n\n{output}\n"
        )


# ──────────────────────────────────────────────────────────────
# Story Discovery
# ──────────────────────────────────────────────────────────────
def discover_stories() -> list[Path]:
    if not STORIES_DIR.exists():
        print(f"{C.FAIL}Stories directory not found: {STORIES_DIR}{C.R}")
        sys.exit(1)
    stories = sorted(STORIES_DIR.glob("story-*.md"))
    if not stories:
        print(f"{C.FAIL}No story files found in {STORIES_DIR}{C.R}")
        sys.exit(1)
    return stories


def extract_tasks(story_file: Path) -> list[str]:
    content = story_file.read_text()
    tasks = [l.strip() for l in content.split("\n") if l.strip().startswith("- [ ]")]
    return tasks or ["Implement full story"]


def story_id_from_path(path: Path) -> str:
    parts = path.stem.split("-")
    return parts[1] if len(parts) >= 2 else path.stem


# ──────────────────────────────────────────────────────────────
# Building Phase
# ──────────────────────────────────────────────────────────────
def build_story(story_file: Path, dev: Agent, qa: Agent, max_rounds: int) -> bool:
    story_content = story_file.read_text()
    tasks = extract_tasks(story_file)
    story_id = story_id_from_path(story_file)

    print(f"{C.SYS}  {len(tasks)} tasks in this story{C.R}\n")

    # Dev implements
    agent_banner("dev", dev.model, f"Implementing story {story_id}...")
    dev_prompt = (
        f"Implement this story. Read the story carefully, then implement ALL tasks in order.\n\n"
        f"STORY:\n```markdown\n{story_content}\n```\n\n"
        f"Start by reading the architecture doc, then explore existing plugins for patterns, "
        f"then implement each task. Run cargo check when done."
    )
    dev_out = dev.run(dev_prompt)

    # QA reviews
    for rnd in range(1, max_rounds + 1):
        agent_banner("qa", qa.model, f"Review round {rnd}/{max_rounds}...")
        qa_prompt = (
            f"Review the implementation of this story:\n\n"
            f"```markdown\n{story_content}\n```\n\n"
            f"Read all source files in default-plugins/session-intelligence/src/, "
            f"check each acceptance criterion, run cargo check, and output your GATE verdict."
        )
        qa_out = qa.run(qa_prompt, context=dev_out[-5000:])

        gate = parse_gate(qa_out)
        gate_banner(gate)

        if gate == "PASS":
            return True

        if rnd < max_rounds:
            agent_banner("dev", dev.model, f"Fixing QA issues (round {rnd})...")
            fix_prompt = (
                f"QA found issues with story {story_id}. Fix ALL required fixes:\n\n"
                f"{qa_out[-4000:]}\n\n"
                f"Story:\n```markdown\n{story_content}\n```\n\n"
                f"Fix the issues and run cargo check to verify."
            )
            dev_out = dev.run(fix_prompt)

    return False


def parse_gate(output: str) -> str:
    upper = output.upper()
    for status in ["PASS", "FAIL", "CONCERNS"]:
        if f"GATE: {status}" in upper or f"GATE:{status}" in upper:
            return status
    if "PASS" in upper and "FAIL" not in upper:
        return "PASS"
    if "FAIL" in upper:
        return "FAIL"
    return "CONCERNS"


def run_pipeline(stories: list[Path], dev: Agent, qa: Agent, max_rounds: int) -> bool:
    phase_banner("PHASE 3: BUILDING", f"Dev/QA pair programming {len(stories)} stories.")

    results = []
    for si, story_file in enumerate(stories, 1):
        story_id = story_id_from_path(story_file)
        print(f"\n{C.BOLD}{C.SYS}")
        print(f"{'=' * 70}")
        print(f"  STORY {si}/{len(stories)}: {story_id} - {story_file.stem}")
        print(f"{'=' * 70}")
        print(f"{C.R}\n")

        result = build_story(story_file, dev, qa, max_rounds)
        results.append((story_id, result))

    # Summary
    passed = sum(1 for _, r in results if r)
    total = len(results)

    print(f"\n{C.BOLD}")
    if passed == total:
        print(f"{C.PASS}")
        print(f"{'=' * 70}")
        print(f"  ALL STORIES COMPLETE")
        print(f"  {total}/{total} stories built and approved by QA.")
        print(f"  Code: default-plugins/session-intelligence/")
        print(f"  Logs: _bmad-output/pipeline-logs/")
        print(f"{'=' * 70}")
    else:
        print(f"{C.WARN}")
        print(f"{'=' * 70}")
        print(f"  PIPELINE NEEDS REVIEW")
        print(f"  {passed}/{total} stories passed. {total - passed} need attention.")
        print(f"  Logs: _bmad-output/pipeline-logs/")
        print(f"{'=' * 70}")
        print(f"\n  Results:")
        for sid, result in results:
            sc = C.PASS if result else C.FAIL
            st = "PASS" if result else "NEEDS REVIEW"
            print(f"    {sc}Story {sid}: {st}{C.R}")
    print(C.R)
    return passed == total


# ──────────────────────────────────────────────────────────────
# CLI
# ──────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(
        description="BMAD Pipeline: Dev/QA pair programming via Anthropic API",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3.11 bmad_prompt2product.py                     # All stories
  python3.11 bmad_prompt2product.py --story 1.1          # Single story
  python3.11 bmad_prompt2product.py --start-from 2.1     # Resume
  python3.11 bmad_prompt2product.py --dry-run             # Preview
  python3.11 bmad_prompt2product.py --builder sonnet      # Cheaper dev
        """
    )
    parser.add_argument("--builder", default="opus",
                        help="Model for Dev agent (default: opus)")
    parser.add_argument("--reviewer", default="haiku",
                        help="Model for QA agent (default: haiku)")
    parser.add_argument("--max-rounds", type=int, default=3,
                        help="Max review rounds per story (default: 3)")
    parser.add_argument("--story", type=str,
                        help="Build a single story by ID (e.g., 1.1)")
    parser.add_argument("--start-from", type=str,
                        help="Start from a specific story ID (e.g., 2.1)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would execute without running")
    args = parser.parse_args()

    # Check API key
    if not os.environ.get("ANTHROPIC_API_KEY") and not args.dry_run:
        print(f"{C.FAIL}ANTHROPIC_API_KEY environment variable not set.{C.R}")
        sys.exit(1)

    # Discover stories
    all_stories = discover_stories()

    # Filter
    if args.story:
        stories = [s for s in all_stories if story_id_from_path(s) == args.story]
        if not stories:
            print(f"{C.FAIL}Story {args.story} not found. Available:")
            for s in all_stories:
                print(f"  {story_id_from_path(s)}: {s.name}")
            print(C.R)
            sys.exit(1)
    elif args.start_from:
        start_idx = next(
            (i for i, s in enumerate(all_stories) if story_id_from_path(s) == args.start_from),
            None
        )
        if start_idx is None:
            print(f"{C.FAIL}Story {args.start_from} not found.{C.R}")
            sys.exit(1)
        stories = all_stories[start_idx:]
    else:
        stories = all_stories

    # Display info
    builder_model = resolve_model(args.builder)
    reviewer_model = resolve_model(args.reviewer)

    print(f"{C.BOLD}{C.SYS}")
    print(f"\u2554{'═' * 58}\u2557")
    print(f"\u2551   BMAD Pipeline: Zellij Session Intelligence              \u2551")
    print(f"\u2551   via Anthropic API (agentic tool use)                    \u2551")
    print(f"\u2551                                                          \u2551")
    print(f"\u2551   Builder:  {builder_model:<45} \u2551")
    print(f"\u2551   Reviewer: {reviewer_model:<45} \u2551")
    print(f"\u2551   Stories:  {len(stories):<45} \u2551")
    print(f"\u2551   Rounds:   {args.max_rounds:<45} \u2551")
    print(f"\u2551                                                          \u2551")
    print(f"\u255a{'═' * 58}\u255d")
    print(C.R)

    print(f"{C.SYS}Stories to build:{C.R}")
    for s in stories:
        sid = story_id_from_path(s)
        tasks = extract_tasks(s)
        print(f"  {sid}: {s.stem} ({len(tasks)} tasks)")
    print()

    if args.dry_run:
        print(f"{C.WARN}Dry run complete. No API calls made.{C.R}")
        return

    # Create client and agents
    client = anthropic.Anthropic()
    dev = Agent("dev", args.builder, client)
    qa = Agent("qa", args.reviewer, client)

    print(f"{C.BOLD}Ready to start autonomous building.{C.R}")
    print(f"{C.SYS}Press Enter to begin, Ctrl+C to abort...{C.R}")
    try:
        input()
    except KeyboardInterrupt:
        print(f"\n{C.SYS}Aborted.{C.R}")
        sys.exit(0)

    try:
        success = run_pipeline(stories, dev, qa, args.max_rounds)
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print(f"\n{C.SYS}Pipeline paused. Logs in _bmad-output/pipeline-logs/{C.R}")
        print(f"{C.SYS}Resume: python3.11 bmad_prompt2product.py --start-from <story-id>{C.R}")
        sys.exit(1)


if __name__ == "__main__":
    main()
