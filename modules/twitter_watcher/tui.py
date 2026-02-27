"""TUI dashboard for the Twitter Watcher module."""

from __future__ import annotations

from typing import Any

from starkbot_sdk.tui import StarkbotDashboard

from rich.console import Group, RenderableType
from rich.panel import Panel
from rich.table import Table
from rich.text import Text


class TwitterWatcherDashboard(StarkbotDashboard):

    def _get_watched_users(self) -> list[dict]:
        """Fetch watched users via the list action."""
        try:
            resp = self.api("/rpc/twitter_watcher", {"action": "list"})
            return sorted(
                resp.get("data", {}).get("entries", []),
                key=lambda e: e["username"].lower(),
            )
        except Exception:
            return []

    def _get_poll_interval(self) -> int:
        try:
            resp = self.api("/rpc/twitter_watcher", {"action": "list"})
            return resp.get("data", {}).get("poll_interval", 120)
        except Exception:
            return 120

    def _get_entry_count(self) -> int:
        return len(self._get_watched_users())

    def build(self, width: int, state: dict | None = None) -> RenderableType:
        users = self._get_watched_users()
        selected = state.get("selected", -1) if state else -1
        scroll = state.get("scroll", 0) if state else 0

        # Clamp selected
        if users and selected >= len(users):
            selected = len(users) - 1

        try:
            status_resp = self.api("/rpc/status")
            uptime = status_resp.get("data", {}).get("uptime_seconds", 0)
        except Exception:
            uptime = 0

        poll_interval = self._get_poll_interval()

        # Format uptime
        mins, secs = divmod(int(uptime), 60)
        hours, mins = divmod(mins, 60)
        if hours:
            uptime_str = f"{hours}h {mins}m {secs}s"
        elif mins:
            uptime_str = f"{mins}m {secs}s"
        else:
            uptime_str = f"{secs}s"

        # Header
        header_text = Text()
        header_text.append("Twitter Watcher", style="bold cyan")
        header_text.append("  |  ", style="dim")
        header_text.append(f"{len(users)}", style="bold green")
        header_text.append(" accounts", style="green")
        header_text.append("  |  ", style="dim")
        header_text.append("polling every ", style="dim")
        header_text.append(f"{poll_interval}s", style="yellow")
        header_text.append("  |  ", style="dim")
        header_text.append("uptime ", style="dim")
        header_text.append(uptime_str, style="yellow")

        header = Panel(header_text, border_style="bright_blue", padding=(0, 1))

        # Visible window
        max_visible = max(1, 20)
        visible_users = users[scroll : scroll + max_visible]

        # Table
        table = Table(
            show_header=True,
            header_style="bold bright_blue",
            border_style="bright_black",
            expand=True,
            pad_edge=True,
        )
        table.add_column("#", style="dim", width=4)
        table.add_column("Username", style="cyan", ratio=1)
        table.add_column("Last Tweet", style="white", ratio=1)
        table.add_column("Status", style="green", ratio=1)

        if users:
            for i, user in enumerate(visible_users):
                row_idx = scroll + i
                username = f"@{user['username']}"
                since_id = user.get("since_id")
                last_tweet = since_id if since_id else "—"
                status = "tracking" if since_id else "seeding"

                idx_str = str(row_idx)
                uname_str = username
                tweet_str = last_tweet
                status_str = status

                if row_idx == selected:
                    idx_str = f"[reverse] {idx_str} [/reverse]"
                    uname_str = f"[reverse]{username}[/reverse]"
                    tweet_str = f"[reverse]{last_tweet}[/reverse]"
                    status_str = f"[reverse]{status}[/reverse]"

                table.add_row(idx_str, uname_str, tweet_str, status_str)
        else:
            table.add_row("", "[dim]No accounts[/dim]", "[dim]—[/dim]", "[dim]—[/dim]")

        # Scroll indicator
        if len(users) > max_visible:
            scroll_text = Text(
                f"  Showing {scroll + 1}-{min(scroll + max_visible, len(users))} of {len(users)}",
                style="dim",
            )
        else:
            scroll_text = Text()

        # Footer with keybindings
        interactive = state is not None
        if interactive:
            footer = Text()
            footer.append("  ↑↓", style="bold white")
            footer.append(" navigate  ", style="dim")
            footer.append("a", style="bold green")
            footer.append(" add  ", style="dim")
            footer.append("d", style="bold red")
            footer.append(" delete  ", style="dim")
            footer.append("r", style="bold cyan")
            footer.append(" refresh  ", style="dim")
            footer.append("q", style="bold white")
            footer.append(" quit", style="dim")
        else:
            footer = Text("  q: quit  |  Ctrl+C: exit", style="dim")

        return Group(header, table, scroll_text, footer)

    def actions(self) -> dict[str, Any]:
        return {
            "navigable": True,
            "actions": [
                {
                    "key": "a",
                    "label": "Add account",
                    "action": "add_account",
                    "prompts": ["Twitter username (without @):"],
                },
                {
                    "key": "d",
                    "label": "Delete",
                    "action": "delete_selected",
                    "confirm": True,
                },
                {
                    "key": "r",
                    "label": "Refresh",
                    "action": "refresh",
                },
            ],
        }

    def handle_action(
        self, action: str, state: dict, inputs: list[str] | None = None
    ) -> dict[str, Any]:
        users = self._get_watched_users()
        selected = state.get("selected", 0)

        if action == "refresh":
            return {"ok": True}

        if action == "add_account":
            if not inputs or len(inputs) < 1 or not inputs[0].strip():
                return {"ok": False, "error": "Username required"}
            username = inputs[0].strip().lstrip("@")
            try:
                resp = self.api("/rpc/twitter_watcher", {"action": "add", "username": username})
                msg = resp.get("data", {}).get("message", f"Added @{username}")
                return {"ok": True, "message": msg}
            except Exception as e:
                return {"ok": False, "error": str(e)}

        if action == "delete_selected":
            if not users or selected < 0 or selected >= len(users):
                return {"ok": False, "error": "No account selected"}
            username = users[selected]["username"]
            self.api("/rpc/twitter_watcher", {"action": "remove", "username": username})
            return {"ok": True, "message": f"Removed @{username}"}

        return {"ok": False, "error": f"Unknown action: {action}"}
