# Story 2.1: Timer-Based Summarization Scheduling

## Epic
Epic 2: AI-Powered Session Summaries

## User Story
As a developer,
I want the plugin to fire a summarization cycle on a configurable interval,
So that summaries stay current without any manual action.

## Requirements
- FR6: System triggers summarization on a configurable schedule

## Tasks
- [ ] In load() or after config is loaded, call set_timeout(config.summarization_interval_secs) to start the timer chain
- [ ] In update(), handle Event::Timer -- this marks the start of a summarization cycle
- [ ] At the end of the Timer handler, re-call set_timeout() to maintain the periodic schedule
- [ ] Use the interval value from PluginConfig (default 120 seconds)

## Acceptance Criteria
- set_timeout is called with the configured interval on plugin start
- On each Timer event, a summarization cycle begins
- set_timeout is re-called at the end of each Timer handler to maintain the periodic schedule
- The interval value comes from PluginConfig

## Technical Context
- set_timeout(seconds: f64) fires a Timer(f64) event after the specified delay
- Timer is chainable -- call set_timeout again at the end of each Timer handler
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
