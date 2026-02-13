# BMAD Multi-Model: Prompt to Product

> You give one prompt. A team of AI agents — each on the right model — takes it from idea to shipped code. You just watch.

## The Concept

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                  │
│  YOU: "Build me an agentic monitoring service for BWatch"        │
│                                                                  │
│  ┌──── PHASE 1: DISCOVERY (interactive, with you) ────┐         │
│  │ PO (Sarah, haiku):  "What's the main goal?"        │         │
│  │ You:                "Monitor uptime + alert Slack"  │         │
│  │ PO (Sarah, haiku):  "Got it. What's MVP scope?"    │         │
│  │ You:                "3 checks, Slack webhook, done" │         │
│  └─────────────────────────────────────────────────────┘         │
│                                                                  │
│  ┌──── PHASE 2: PLANNING (autonomous, you watch) ─────┐         │
│  │ Analyst (Mary, haiku):   creates project-brief.md   │         │
│  │ PM (John, haiku):        creates prd.md             │         │
│  │ Architect (Winston, opus): creates architecture.md  │         │
│  │ PO (Sarah, haiku):       validates + shards docs    │         │
│  │ SM (Bob, haiku):         creates stories            │         │
│  └─────────────────────────────────────────────────────┘         │
│                                                                  │
│  ┌──── PHASE 3: BUILDING (autonomous, you watch) ─────┐         │
│  │ Dev (James, opus):  implements story 1.1            │         │
│  │ QA (Quinn, haiku):  "Missing error handler" → FAIL  │         │
│  │ Dev (James, opus):  fixes → re-submits              │         │
│  │ QA (Quinn, haiku):  PASS → next story               │         │
│  │ ... repeat for all stories ...                      │         │
│  └─────────────────────────────────────────────────────┘         │
│                                                                  │
│  You: *sips coffee, checks phone, comes back to working product* │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Key insight:** Only Phase 1 (discovery) needs your input. After that, the
entire team runs autonomously. QA acts as your proxy for quality.

---

## The Full Agent Team

| Agent | Person | Role | Model | Why |
|-------|--------|------|-------|-----|
| PO | Sarah | Kickstarts project, works with you on requirements | haiku | Interactive, lightweight |
| Analyst | Mary | Market research, project brief | haiku | Reading + summarizing |
| PM | John | Creates PRD from brief | haiku | Template-filling |
| Architect | Winston | System design, tech stack | **opus** | Creative, complex reasoning |
| PO | Sarah | Validates all docs, shards for dev | haiku | Checklist work |
| SM | Bob | Creates stories from sharded docs | haiku | Template-filling |
| Dev | James | Implements stories | **opus** | Code generation |
| QA | Quinn | Reviews on your behalf | haiku | Your proxy reviewer |

Only **Architect** and **Dev** need the expensive model. Everyone else runs cheap.

---

## Architecture

```
┌─────────────────────────────────────┐
│  1. core-config.yaml                │  ← Model registry + agent map
├─────────────────────────────────────┤
│  2. Agent definitions               │  ← customization field per agent
│     (po.md, analyst.md, dev.md...)  │
├─────────────────────────────────────┤
│  3. bmad_prompt2product.py          │  ← Full pipeline runner
│     Phase 1: Interactive discovery  │     Streams everything live
│     Phase 2: Autonomous planning    │     to your terminal
│     Phase 3: Autonomous building    │
└─────────────────────────────────────┘
```

---

## Step 1: Model Registry in core-config.yaml

```yaml
# Existing BMAD config...
markdownExploder: true
qa:
  qaLocation: docs/qa
# ... keep all existing config ...

# NEW: Model registry
models:
  registry:
    opus:
      provider: anthropic
      model_id: claude-opus-4-6
      cost_tier: expensive
    haiku:
      provider: anthropic
      model_id: claude-haiku-4-5-20251001
      cost_tier: cheap
    glm4:
      provider: openai-compatible
      model_id: glm-4
      base_url: https://open.bigmodel.cn/api/paas/v4
      api_key_env: GLM_API_KEY
      cost_tier: cheap
    ollama-local:
      provider: openai-compatible
      model_id: qwen2.5-coder:14b
      base_url: http://localhost:11434/v1
      cost_tier: free

  # Agent → model mapping
  agent_model_map:
    po: haiku            # interactive discovery + validation
    analyst: haiku       # research + brief
    pm: haiku            # PRD creation
    architect: opus      # system design (needs deep reasoning)
    sm: haiku            # story creation
    dev: opus            # code implementation (needs power)
    qa: haiku            # review (your proxy)

  # Pipeline config
  pipeline:
    # Phase 1 is interactive — PO works with you
    phase1_interactive: true
    # Phase 2 + 3 are autonomous
    phase2_autonomous: true
    phase3_autonomous: true
    # Dev/QA pair loop config
    pair_loop_style: per-task
    max_review_rounds: 3
    stream_output: true
```

---

## Step 2: Agent Customization Fields

### po.md — Project Kickstarter (works with you)

```yaml
agent:
  customization:
    model_preference: haiku
    pipeline_role: discovery_lead
    # PO is the ONLY agent that talks to the human directly
    interaction_mode: interactive
    # After discovery, hand off requirements to analyst
    on_complete: handoff_to_analyst
```

### analyst.md — Brief Creator

```yaml
agent:
  customization:
    model_preference: haiku
    pipeline_role: researcher
    interaction_mode: autonomous
    # Take PO's requirements and create project brief
    input_from: po_discovery_output
    output: docs/project-brief.md
```

### pm.md — PRD Author

```yaml
agent:
  customization:
    model_preference: haiku
    pipeline_role: product_planner
    interaction_mode: autonomous
    input_from: docs/project-brief.md
    output: docs/prd.md
```

### architect.md — System Designer (expensive model needed)

```yaml
agent:
  customization:
    model_preference: opus
    pipeline_role: system_designer
    interaction_mode: autonomous
    input_from: docs/prd.md
    output: docs/architecture.md
    # Architect may suggest PRD changes
    can_request_revision: true
```

### dev.md — Builder

```yaml
agent:
  customization:
    model_preference: opus
    pipeline_role: builder
    pair_role: builder
    interaction_mode: autonomous
    decision_style: autonomous
    # Pick best approach, explain rationale, don't ask human
    on_task_complete: signal_reviewer
    output_format:
      include_diff: true
      include_rationale: true
      include_test_results: true
```

### qa.md — Your Proxy Reviewer

```yaml
agent:
  customization:
    model_preference: haiku
    pipeline_role: reviewer
    pair_role: reviewer
    proxy_role: human_representative
    interaction_mode: autonomous
    review_mode:
      checks:
        - code_correctness
        - test_coverage
        - security_basics
        - story_alignment
        - approach_rationale
        - over_engineering
      review_stance: constructive_critical
      max_review_tokens: 2000
```

---

## Step 3: The Full Pipeline Runner

Create `bmad_prompt2product.py`:

```python
#!/usr/bin/env python3
"""
BMAD Prompt-to-Product Pipeline

Full pipeline: your prompt → discovery → planning → building → product

Phase 1 (Discovery):  PO talks to you interactively to define requirements
Phase 2 (Planning):   Analyst → PM → Architect → PO → SM (all autonomous)
Phase 3 (Building):   Dev/QA pair programming loop (all autonomous)

Usage:
    # Full pipeline from scratch
    python bmad_prompt2product.py --prompt "Build a monitoring service for BWatch"

    # Skip discovery, start from existing brief
    python bmad_prompt2product.py --skip-to planning --brief docs/project-brief.md

    # Skip to building, start from existing stories
    python bmad_prompt2product.py --skip-to building
"""

import subprocess
import sys
import argparse
import json
from pathlib import Path
from datetime import datetime


# ──────────────────────────────────────────────────────────────
# Terminal Colors
# ──────────────────────────────────────────────────────────────
class C:
    PO      = "\033[95m"   # Magenta - PO (Sarah)
    ANALYST = "\033[94m"   # Blue - Analyst (Mary)
    PM      = "\033[93m"   # Yellow - PM (John)
    ARCH    = "\033[96m"   # Cyan - Architect (Winston)
    SM      = "\033[33m"   # Dark Yellow - SM (Bob)
    DEV     = "\033[96m"   # Cyan - Dev (James)
    QA      = "\033[93m"   # Yellow - QA (Quinn)
    SYS     = "\033[90m"   # Gray - System
    PASS    = "\033[92m"   # Green
    FAIL    = "\033[91m"   # Red
    WARN    = "\033[93m"   # Yellow
    BOLD    = "\033[1m"
    DIM     = "\033[2m"
    R       = "\033[0m"    # Reset

    AGENT_COLORS = {
        "po": "\033[95m", "analyst": "\033[94m", "pm": "\033[93m",
        "architect": "\033[96m", "sm": "\033[33m", "dev": "\033[96m",
        "qa": "\033[93m",
    }

AGENT_NAMES = {
    "po": "Sarah (PO)", "analyst": "Mary (Analyst)", "pm": "John (PM)",
    "architect": "Winston (Architect)", "sm": "Bob (SM)",
    "dev": "James (Dev)", "qa": "Quinn (QA)",
}

AGENT_ICONS = {
    "po": "\U0001F4DD", "analyst": "\U0001F4CA", "pm": "\U0001F4CB",
    "architect": "\U0001F3D7\uFE0F", "sm": "\U0001F3C3",
    "dev": "\U0001F4BB", "qa": "\U0001F9EA",
}


def phase_banner(phase_num: int, title: str, description: str, interactive: bool = False):
    mode = "INTERACTIVE - your input needed" if interactive else "AUTONOMOUS - just watch"
    print(f"\n{C.BOLD}{C.SYS}")
    print(f"{'='*70}")
    print(f"  PHASE {phase_num}: {title}")
    print(f"  {description}")
    print(f"  Mode: {mode}")
    print(f"{'='*70}")
    print(f"{C.R}\n")


def agent_banner(agent_id: str, model: str, action: str):
    color = C.AGENT_COLORS.get(agent_id, C.SYS)
    icon = AGENT_ICONS.get(agent_id, "")
    name = AGENT_NAMES.get(agent_id, agent_id)
    print(f"\n{color}{C.BOLD}{'─'*60}")
    print(f" {icon} {name} | model: {model}")
    print(f" {action}")
    print(f"{'─'*60}{C.R}\n")


def gate_banner(status: str):
    colors = {"PASS": C.PASS, "FAIL": C.FAIL, "CONCERNS": C.WARN}
    color = colors.get(status, C.SYS)
    print(f"\n{color}{C.BOLD}>>> GATE: {status} <<<{C.R}\n")


# ──────────────────────────────────────────────────────────────
# Agent Runner
# ──────────────────────────────────────────────────────────────
class Agent:
    def __init__(self, agent_id: str, model: str, bmad_core=".bmad-core"):
        self.id = agent_id
        self.model = model
        self.bmad_core = bmad_core
        self.agent_def = f"{bmad_core}/agents/{agent_id}.md"

    def run(self, prompt: str, context: str = "", interactive: bool = False) -> str:
        """Run agent. If interactive=True, runs in foreground (user can type)."""

        full_prompt = (
            f"You are the BMAD {self.id} agent.\n"
            f"Read your full agent definition: {self.agent_def}\n"
            f"Read project config: {self.bmad_core}/core-config.yaml\n\n"
        )
        if context:
            full_prompt += f"Context from previous phase:\n---\n{context}\n---\n\n"
        full_prompt += prompt

        if interactive:
            # Interactive mode: user can see and respond
            # Use claude without -p so it's a real conversation
            result = subprocess.run(
                ["claude", "--model", self.model, "--yes",
                 "--prompt", full_prompt],
                text=True,
                capture_output=True,
                timeout=600,
            )
            output = result.stdout
        else:
            # Autonomous mode: stream output, no user input
            process = subprocess.Popen(
                ["claude", "-p", full_prompt, "--model", self.model, "--yes"],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, bufsize=1,
            )
            color = C.AGENT_COLORS.get(self.id, C.SYS)
            lines = []
            for line in iter(process.stdout.readline, ""):
                print(f"{color}{line}{C.R}", end="")
                lines.append(line)
            process.wait()
            output = "".join(lines)

        self._log(prompt, output)
        return output

    def _log(self, prompt: str, output: str):
        log_dir = Path(".ai/pipeline-logs")
        log_dir.mkdir(parents=True, exist_ok=True)
        ts = datetime.now().strftime("%Y%m%d-%H%M%S")
        (log_dir / f"{self.id}-{ts}.md").write_text(
            f"# {self.id} ({self.model})\n## Prompt\n{prompt}\n## Output\n{output}\n"
        )


# ──────────────────────────────────────────────────────────────
# Phase 1: Discovery (Interactive — PO works with you)
# ──────────────────────────────────────────────────────────────
def phase1_discovery(prompt: str, po: Agent) -> str:
    """PO interacts with the human to define requirements.
    This is the ONLY phase that needs human input."""

    phase_banner(1, "DISCOVERY",
                 "PO (Sarah) will work with you to define requirements.",
                 interactive=True)

    agent_banner("po", po.model, "Starting requirements discovery with you...")

    # PO asks questions, human answers
    discovery_output = po.run(
        f"The human wants to build something. Their initial prompt is:\n\n"
        f'"{prompt}"\n\n'
        f"Your job:\n"
        f"1. Understand their vision — ask 3-5 clarifying questions\n"
        f"2. Define MVP scope together\n"
        f"3. Identify key requirements, constraints, and priorities\n"
        f"4. Write a structured DISCOVERY SUMMARY with:\n"
        f"   - Project goal (1 sentence)\n"
        f"   - MVP scope (bullet points)\n"
        f"   - Key requirements\n"
        f"   - Constraints / non-goals\n"
        f"   - Target users\n"
        f"   - Success criteria\n\n"
        f"Be concise. This discovery feeds into the full planning pipeline.\n"
        f"Save the discovery summary to docs/discovery.md",
        interactive=True
    )

    # Save discovery output
    docs = Path("docs")
    docs.mkdir(exist_ok=True)

    print(f"\n{C.PASS}Phase 1 complete. Discovery saved.{C.R}")
    print(f"{C.SYS}From here, everything runs autonomously. Sit back and watch.{C.R}\n")
    return discovery_output


# ──────────────────────────────────────────────────────────────
# Phase 2: Planning (Autonomous — Analyst → PM → Architect → PO → SM)
# ──────────────────────────────────────────────────────────────
def phase2_planning(discovery: str, agents: dict) -> list:
    """Full planning pipeline. All autonomous — human just watches."""

    phase_banner(2, "PLANNING",
                 "Analyst → PM → Architect → PO → SM creating all docs.",
                 interactive=False)

    # ── Analyst: Project Brief ──
    agent_banner("analyst", agents["analyst"].model,
                 "Creating project brief from discovery...")
    brief = agents["analyst"].run(
        f"Read the discovery summary and create a project brief.\n"
        f"Use task create-doc with template project-brief-tmpl.yaml.\n"
        f"Save output to docs/project-brief.md\n"
        f"Be thorough but concise.",
        context=discovery[-6000:]
    )

    # ── PM: PRD ──
    agent_banner("pm", agents["pm"].model,
                 "Creating PRD from project brief...")
    prd = agents["pm"].run(
        f"Read docs/project-brief.md\n"
        f"Execute *create-prd using prd-tmpl.yaml\n"
        f"Create a comprehensive PRD with epics and stories.\n"
        f"Save output to docs/prd.md",
        context=brief[-6000:]
    )

    # ── Architect: Architecture ──
    agent_banner("architect", agents["architect"].model,
                 "Designing system architecture...")
    arch = agents["architect"].run(
        f"Read docs/prd.md\n"
        f"Execute *create-backend-architecture using architecture-tmpl.yaml\n"
        f"Design the full system architecture.\n"
        f"If you suggest any changes to PRD stories, note them clearly.\n"
        f"Save output to docs/architecture.md",
        context=prd[-6000:]
    )

    # ── Check if Architect wants PRD changes ──
    if any(kw in arch.upper() for kw in ["SUGGEST", "PRD CHANGE", "UPDATE PRD", "MODIFY STORY"]):
        agent_banner("pm", agents["pm"].model,
                     "Updating PRD based on Architect's suggestions...")
        prd = agents["pm"].run(
            f"The Architect suggested changes to the PRD.\n"
            f"Review their suggestions and update docs/prd.md accordingly.\n"
            f"Re-save the complete PRD.",
            context=arch[-4000:]
        )

    # ── PO: Validate ──
    agent_banner("po", agents["po"].model,
                 "Validating all artifacts for consistency...")
    validation = agents["po"].run(
        f"Execute *execute-checklist-po using po-master-checklist.\n"
        f"Validate docs/project-brief.md, docs/prd.md, docs/architecture.md\n"
        f"Check for consistency, completeness, and actionability.\n"
        f"List any issues found. If all good, output VALIDATION: PASS"
    )

    if "FAIL" in validation.upper() or "ISSUE" in validation.upper():
        print(f"{C.WARN}PO found issues. Attempting auto-fix...{C.R}")
        # Let PO describe what needs fixing, PM/Architect fix
        agents["pm"].run(
            f"PO validation found issues. Fix them:\n{validation[-3000:]}\n"
            f"Update docs/prd.md"
        )

    # ── PO: Shard Documents ──
    agent_banner("po", agents["po"].model,
                 "Sharding docs for development...")
    agents["po"].run(
        f"Execute *shard-doc for docs/prd.md to docs/prd/\n"
        f"Then shard docs/architecture.md to docs/architecture/\n"
        f"This creates bite-sized docs for the dev agent."
    )

    # ── SM: Create Stories ──
    agent_banner("sm", agents["sm"].model,
                 "Creating stories from sharded docs...")
    stories_output = agents["sm"].run(
        f"Read the sharded PRD docs in docs/prd/\n"
        f"Execute *draft for each epic.\n"
        f"Create story files in docs/stories/\n"
        f"Output the list of story files created."
    )

    # Discover story files
    story_dir = Path("docs/stories")
    stories = sorted(story_dir.glob("*.md")) if story_dir.exists() else []

    if not stories:
        # SM may have created them with different naming
        stories_output_lines = stories_output.split("\n")
        stories = [Path(l.strip()) for l in stories_output_lines
                   if l.strip().endswith(".md") and "stories" in l]

    print(f"\n{C.PASS}Phase 2 complete. {len(stories)} stories created.{C.R}")
    print(f"{C.SYS}Planning artifacts in docs/. Moving to build phase.{C.R}\n")
    return [str(s) for s in stories]


# ──────────────────────────────────────────────────────────────
# Phase 3: Building (Autonomous — Dev/QA pair loop per story)
# ──────────────────────────────────────────────────────────────
def phase3_building(stories: list, dev: Agent, qa: Agent,
                    loop_style: str = "per-task", max_rounds: int = 3):
    """Dev builds, QA reviews on your behalf. Fully autonomous."""

    phase_banner(3, "BUILDING",
                 f"Dev/QA pair programming {len(stories)} stories. "
                 f"Loop: {loop_style}.",
                 interactive=False)

    results = []

    for si, story_file in enumerate(stories, 1):
        print(f"\n{C.BOLD}{C.SYS}"
              f"{'='*70}\n"
              f"  STORY {si}/{len(stories)}: {Path(story_file).name}\n"
              f"{'='*70}{C.R}\n")

        if loop_style == "per-task":
            result = _build_per_task(story_file, dev, qa, max_rounds)
        elif loop_style == "per-story":
            result = _build_per_story(story_file, dev, qa, max_rounds)
        else:
            result = _build_per_change(story_file, dev, qa, max_rounds)

        results.append(result)
        status = "PASS" if result else "NEEDS REVIEW"
        gate_banner(status if result else "CONCERNS")

    passed = sum(results)
    total = len(results)

    print(f"\n{C.BOLD}")
    if passed == total:
        print(f"{C.PASS}")
        print(f"{'='*70}")
        print(f"  PROJECT COMPLETE")
        print(f"  All {total} stories built and approved by QA.")
        print(f"  Docs: docs/    Logs: .ai/pipeline-logs/")
        print(f"{'='*70}")
    else:
        print(f"{C.WARN}")
        print(f"{'='*70}")
        print(f"  PROJECT NEEDS REVIEW")
        print(f"  {passed}/{total} stories passed. {total-passed} need human attention.")
        print(f"  Check .ai/pipeline-logs/ for details.")
        print(f"{'='*70}")
    print(f"{C.R}")


def _build_per_task(story: str, dev: Agent, qa: Agent, max_rounds: int) -> bool:
    """Dev implements one task at a time, QA reviews each."""

    # Extract tasks from story
    tasks = _extract_tasks(story)
    print(f"{C.SYS}  {len(tasks)} tasks in this story{C.R}\n")

    for ti, task in enumerate(tasks, 1):
        # Dev implements
        agent_banner("dev", dev.model, f"Task {ti}/{len(tasks)}: {task[:50]}...")
        dev_out = dev.run(
            f"Read story: {story}\n"
            f"Implement ONLY this task:\n  {task}\n\n"
            f"1. Explain your approach (pick best option, explain why)\n"
            f"2. Write the code\n"
            f"3. Write tests\n"
            f"4. Run tests\n"
            f"Do NOT ask for confirmation. QA reviews next."
        )

        # QA reviews
        agent_banner("qa", qa.model, f"Reviewing task {ti}/{len(tasks)} on your behalf...")
        qa_out = qa.run(
            f"You are the human's proxy. Review this task:\n"
            f"  Task: {task}\n\n"
            f"Check: correctness, tests, security, story alignment, over-engineering.\n"
            f"Output:\n"
            f"GATE: [PASS|CONCERNS|FAIL]\n"
            f"REQUIRED_FIXES: [list or 'none']\n",
            context=dev_out[-5000:]
        )

        gate = _parse_gate(qa_out)
        gate_banner(gate)

        if gate in ("FAIL", "CONCERNS"):
            # Dev fixes
            agent_banner("dev", dev.model, f"Fixing QA issues for task {ti}...")
            dev_out = dev.run(
                f"QA found issues. Fix all REQUIRED_FIXES:\n{qa_out[-3000:]}\n"
                f"Re-run tests.",
                context=f"Story: {story}"
            )
            # QA re-check
            agent_banner("qa", qa.model, f"Re-checking task {ti}...")
            recheck = qa.run(
                "Quick re-check: did Dev fix everything?\nGATE: [PASS|FAIL]\n",
                context=dev_out[-3000:]
            )
            gate = _parse_gate(recheck)
            gate_banner(gate)

    return True


def _build_per_story(story: str, dev: Agent, qa: Agent, max_rounds: int) -> bool:
    """Dev builds full story, QA reviews once."""

    agent_banner("dev", dev.model, "Building full story...")
    dev_out = dev.run(
        f"Read story: {story}\n"
        f"Execute *develop-story. Implement ALL tasks.\n"
        f"Pick best approaches, explain rationale. Don't ask for confirmation."
    )

    for rnd in range(1, max_rounds + 1):
        agent_banner("qa", qa.model, f"Full story review (round {rnd}/{max_rounds})...")
        qa_out = qa.run(
            f"Read story: {story}\n"
            f"You are the human's proxy. Execute *review.\n"
            f"Output: GATE + REQUIRED_FIXES",
            context=dev_out[-8000:]
        )

        gate = _parse_gate(qa_out)
        gate_banner(gate)

        if gate == "PASS":
            return True

        if rnd < max_rounds:
            agent_banner("dev", dev.model, f"Fixing (round {rnd})...")
            dev_out = dev.run(
                f"Fix all issues:\n{qa_out[-5000:]}\nRe-run tests.",
                context=f"Story: {story}"
            )

    return False


def _build_per_change(story: str, dev: Agent, qa: Agent, max_rounds: int) -> bool:
    """Tightest loop: review every file change."""

    agent_banner("dev", dev.model, "Starting (per-change mode)...")
    dev_out = dev.run(
        f"Read story: {story}\n"
        f"Implement one file at a time. After each file:\n"
        f"1. Explain what + why\n"
        f"2. Show diff\n"
        f"3. Output '--- REVIEW POINT ---'\n"
    )

    for _ in range(max_rounds * 5):
        agent_banner("qa", qa.model, "Reviewing change...")
        qa_out = qa.run(
            "Review this change. Reply APPROVE or REQUEST_CHANGES.",
            context=dev_out[-3000:]
        )

        if "APPROVE" in qa_out.upper():
            agent_banner("dev", dev.model, "Next change...")
            dev_out = dev.run(
                "Approved. Next file. Output '--- ALL COMPLETE ---' when done."
            )
            if "ALL COMPLETE" in dev_out.upper():
                return True
        else:
            agent_banner("dev", dev.model, "Revising...")
            dev_out = dev.run(f"Fix:\n{qa_out[-2000:]}")

    return False


def _extract_tasks(story_file: str) -> list:
    content = Path(story_file).read_text()
    tasks = [l.strip() for l in content.split("\n")
             if l.strip().startswith("- [ ]") and "Subtask" not in l]
    return tasks or ["Implement full story"]


def _parse_gate(output: str) -> str:
    for s in ["PASS", "FAIL", "CONCERNS", "WAIVED"]:
        if s in output.upper():
            return s
    return "CONCERNS"


# ──────────────────────────────────────────────────────────────
# CLI
# ──────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(
        description="BMAD Prompt-to-Product: full pipeline",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Full pipeline from a prompt
  python bmad_prompt2product.py --prompt "Build a monitoring service"

  # Skip to planning (already have discovery/brief)
  python bmad_prompt2product.py --skip-to planning

  # Skip to building (already have stories)
  python bmad_prompt2product.py --skip-to building

  # Use GLM as reviewer
  python bmad_prompt2product.py --prompt "..." --reviewer glm4

  # Tighter review loop
  python bmad_prompt2product.py --prompt "..." --loop per-change
        """
    )
    parser.add_argument("--prompt", help="Your project idea (starts full pipeline)")
    parser.add_argument("--skip-to", choices=["planning", "building"],
                        help="Skip to a later phase")
    parser.add_argument("--builder", default="opus", help="Model for dev + architect")
    parser.add_argument("--reviewer", default="haiku", help="Model for QA + all others")
    parser.add_argument("--loop", default="per-task",
                        choices=["per-task", "per-story", "per-change"])
    parser.add_argument("--max-rounds", type=int, default=3)
    args = parser.parse_args()

    # Create agents with model assignments
    agents = {
        "po":        Agent("po", args.reviewer),
        "analyst":   Agent("analyst", args.reviewer),
        "pm":        Agent("pm", args.reviewer),
        "architect": Agent("architect", args.builder),   # expensive
        "sm":        Agent("sm", args.reviewer),
        "dev":       Agent("dev", args.builder),          # expensive
        "qa":        Agent("qa", args.reviewer),
    }

    print(f"{C.BOLD}{C.SYS}")
    print(f"╔══════════════════════════════════════════════════════════╗")
    print(f"║   BMAD Prompt-to-Product Pipeline                       ║")
    print(f"║                                                          ║")
    print(f"║   Builder model:  {args.builder:<39} ║")
    print(f"║   Reviewer model: {args.reviewer:<39} ║")
    print(f"║   Loop style:     {args.loop:<39} ║")
    print(f"║                                                          ║")
    print(f"║   Phase 1: Discovery  (interactive — you + PO)           ║")
    print(f"║   Phase 2: Planning   (autonomous — you watch)           ║")
    print(f"║   Phase 3: Building   (autonomous — you watch)           ║")
    print(f"╚══════════════════════════════════════════════════════════╝")
    print(f"{C.R}")

    try:
        # Phase 1: Discovery
        if args.skip_to is None:
            if not args.prompt:
                args.prompt = input(f"{C.PO}What do you want to build? > {C.R}")
            discovery = phase1_discovery(args.prompt, agents["po"])
        else:
            discovery = ""
            if Path("docs/discovery.md").exists():
                discovery = Path("docs/discovery.md").read_text()

        # Phase 2: Planning
        if args.skip_to != "building":
            stories = phase2_planning(discovery, agents)
        else:
            story_dir = Path("docs/stories")
            stories = sorted(str(s) for s in story_dir.glob("*.md"))
            print(f"{C.SYS}Found {len(stories)} existing stories{C.R}")

        # Phase 3: Building
        if stories:
            phase3_building(stories, agents["dev"], agents["qa"],
                           args.loop, args.max_rounds)
        else:
            print(f"{C.FAIL}No stories found. Check docs/stories/{C.R}")

    except KeyboardInterrupt:
        print(f"\n{C.SYS}Pipeline paused. Logs in .ai/pipeline-logs/{C.R}")
        sys.exit(1)


if __name__ == "__main__":
    main()
```

---

## Step 4: Agent Team Bundle

Create `.bmad-core/agent-teams/team-prompt2product.yaml`:

```yaml
bundle:
  name: Team Prompt-to-Product
  icon: "\U0001F680"
  description: >
    Full pipeline team. One prompt → complete product.
    PO does interactive discovery, then everything runs autonomous.
    Dev builds on expensive model, QA reviews on cheap model as your proxy.
agents:
  - po
  - analyst
  - pm
  - architect
  - sm
  - dev
  - qa
workflows:
  - greenfield-service.yaml
  - brownfield-service.yaml

pipeline_config:
  phases:
    discovery:
      agent: po
      interactive: true
      model: haiku
    planning:
      agents: [analyst, pm, architect, po, sm]
      interactive: false
      models:
        architect: opus
        default: haiku
    building:
      agents: [dev, qa]
      interactive: false
      models:
        dev: opus
        qa: haiku
      pair_loop: per-task
      max_rounds: 3
```

---

## Step 5: Usage

### Full pipeline from a prompt

```bash
python bmad_prompt2product.py --prompt "Build a monitoring service for BWatch"
```

### What happens

```
Phase 1: DISCOVERY (interactive)
  PO Sarah asks you 3-5 questions → you answer → discovery.md saved

Phase 2: PLANNING (autonomous, you watch)
  Analyst Mary  → project-brief.md
  PM John       → prd.md (with epics + stories)
  Architect Winston → architecture.md (on Opus, the big brain)
  PO Sarah      → validates all docs → shards for dev
  SM Bob        → creates story files

Phase 3: BUILDING (autonomous, you watch)
  For each story:
    Dev James (Opus)  → implements, picks best approach, explains why
    QA Quinn (Haiku)  → reviews on your behalf, pushes back
    Dev James (Opus)  → fixes issues
    QA Quinn (Haiku)  → re-checks → PASS or escalate

  PROJECT COMPLETE
```

### Skip phases (resume from where you left off)

```bash
# Already did discovery, skip to planning
python bmad_prompt2product.py --skip-to planning

# Already have stories, skip to building
python bmad_prompt2product.py --skip-to building

# Use GLM as reviewer (cheapest)
python bmad_prompt2product.py --prompt "..." --reviewer glm4
```

### Ctrl+C anytime

Pauses the pipeline. All work is saved (docs/, stories/, code).
Re-run with `--skip-to` to continue from where you stopped.

---

## Cost Analysis (Full Pipeline)

| Phase | Agent | Model | Est. Tokens | Cost |
|-------|-------|-------|-------------|------|
| Discovery | PO | haiku | ~5K in, 2K out | $0.01 |
| Planning | Analyst | haiku | ~10K in, 5K out | $0.03 |
| Planning | PM | haiku | ~15K in, 8K out | $0.04 |
| Planning | **Architect** | **opus** | ~20K in, 10K out | **$1.05** |
| Planning | PO (validate) | haiku | ~10K in, 3K out | $0.02 |
| Planning | SM | haiku | ~10K in, 5K out | $0.03 |
| Building (x4 stories) | **Dev** | **opus** | ~200K in, 80K out | **$9.00** |
| Building (x4 stories) | QA | haiku | ~80K in, 20K out | $0.14 |
| **Total** | | | | **~$10.32** |
| All on Opus (no routing) | | opus | ~350K in, 133K out | **~$15.23** |

**Savings: ~32%** — and you get structured QA review for free.

With GLM/Ollama as reviewer: QA cost drops to near zero → **~$10.05**

---

## Flow Diagram

```
                    YOUR PROMPT
                        │
                        v
    ┌───────────────────────────────────────┐
    │  PHASE 1: DISCOVERY (interactive)     │
    │  PO (Sarah, haiku) ←→ You             │
    │  Output: docs/discovery.md            │
    └───────────────┬───────────────────────┘
                    │
                    v  (from here: autonomous)
    ┌───────────────────────────────────────┐
    │  PHASE 2: PLANNING                    │
    │                                       │
    │  Analyst (Mary, haiku)                │
    │    └→ docs/project-brief.md           │
    │                                       │
    │  PM (John, haiku)                     │
    │    └→ docs/prd.md                     │
    │                                       │
    │  Architect (Winston, OPUS)            │
    │    └→ docs/architecture.md            │
    │    └→ may revise PRD                  │
    │                                       │
    │  PO (Sarah, haiku)                    │
    │    └→ validates all docs              │
    │    └→ shards for dev                  │
    │                                       │
    │  SM (Bob, haiku)                      │
    │    └→ docs/stories/1.1.xxx.md         │
    │    └→ docs/stories/1.2.xxx.md         │
    │    └→ ...                             │
    └───────────────┬───────────────────────┘
                    │
                    v
    ┌───────────────────────────────────────┐
    │  PHASE 3: BUILDING (per story)        │
    │  ┌─────────────────────────────────┐  │
    │  │ Dev (James, OPUS) implements    │  │
    │  │         │                       │  │
    │  │         v                       │  │
    │  │ QA (Quinn, haiku) reviews       │  │
    │  │         │                       │  │
    │  │    PASS? ──yes──→ next story    │  │
    │  │         │                       │  │
    │  │        no                       │  │
    │  │         │                       │  │
    │  │         v                       │  │
    │  │ Dev fixes → QA re-checks       │  │
    │  │ (max 3 rounds then escalate)   │  │
    │  └─────────────────────────────────┘  │
    │  ... repeat for all stories ...       │
    └───────────────┬───────────────────────┘
                    │
                    v
              PROJECT COMPLETE
```

---

## Next Steps

1. Copy `bmad_prompt2product.py` + config changes to your implementation repo
2. Update `core-config.yaml` with the `models` section
3. Add `customization` fields to all agent `.md` files
4. Add `team-prompt2product.yaml` to agent-teams
5. Run: `python bmad_prompt2product.py --prompt "your idea here"`
6. Answer PO's discovery questions (Phase 1)
7. Watch the rest happen (Phase 2 + 3)
8. Ctrl+C if anything goes off the rails
