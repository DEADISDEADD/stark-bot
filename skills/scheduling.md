---
name: scheduling
description: "Create scheduled tasks (cron jobs) that run at specific times or intervals. Use for recurring reports, reminders, and automated tasks."
tags: [cron, schedule, automation, recurring, scheduling, social]
---

# Scheduling Tasks

When a user asks you to do something "every X hours", "daily", "weekly", or at a specific time, you need to CREATE A CRON JOB, not execute the task immediately.

## IMPORTANT: Recognize Scheduling Requests

Keywords that indicate a scheduling request:
- "every X hours/minutes/days"
- "daily", "weekly", "monthly"
- "at 9am", "every morning"
- "schedule", "recurring", "repeat"

When you see these keywords, DO NOT execute the task immediately. Instead, tell the user you'll create a scheduled task and explain what it will do.

## How Cron Jobs Work in StarkBot

Cron jobs are managed through the API (not directly by the agent). To create a cron job:

1. Explain to the user what the scheduled task will do
2. Tell them to use the Scheduling page in the web UI to create it
3. Or provide the cron job configuration they need:

```config:cron_job
name: Daily PR Report
description: Generate a summary of recent PRs and post to Discord
schedule_type: every
schedule_value: "24h"
message: Generate a report of recent PRs on clawdbot/clawdbot and summarize them
channel_id: <discord_channel_id>
deliver: true
```

## Schedule Types

- `at`: Run at specific time (e.g., "09:00")
- `every`: Run at interval (e.g., "1h", "24h", "30m")
- `cron`: Standard cron expression (e.g., "0 9 * * *" for 9am daily)

## Example Responses

User: "Every 24 hours, make a report of PRs on repo X and post to Discord"

Response: "I'll help you set up a scheduled task for that. This needs to be created as a cron job. Here's what you need:

**Cron Job Configuration:**
- Name: Daily PR Report for repo X
- Schedule: Every 24 hours
- Task: Generate PR summary and post to Discord

You can create this in the **Scheduling** page of the StarkBot web UI, or I can describe the exact steps. The cron job will run the report automatically and deliver the results to your Discord channel."

## DO NOT

- Do NOT immediately execute commands when the user asks for recurring tasks
- Do NOT say "I can't schedule things" - explain how cron jobs work
- Do NOT try to use `exec` with `cron` or `at` commands - use the StarkBot scheduling system
