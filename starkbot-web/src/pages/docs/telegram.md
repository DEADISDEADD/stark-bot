---
name: Telegram Integration
---

This guide walks you through setting up a Telegram bot using @BotFather and connecting it to StarkBot.

## Creating Your Bot with BotFather

Telegram's official [@BotFather](https://t.me/BotFather) manages all bot creation and configuration.

### Step 1: Create a New Bot

1. Open Telegram and search for **@BotFather**
2. Start a chat and send `/newbot`
3. Enter a **display name** for your bot (e.g., "My StarkBot")
4. Enter a **username** ending in `bot` (e.g., `mystarkbot_bot`)
5. BotFather will respond with your **bot token**

### Step 2: Save Your Token

The token looks like: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`

**Keep this token secure** — it grants full control over your bot.

> If your token is ever compromised, use `/revoke` in BotFather to regenerate it.

---

## Optional BotFather Settings

Customize your bot's behavior with these commands:

| Command | Description |
|---------|-------------|
| `/setname` | Change display name |
| `/setdescription` | Set bot description |
| `/setabouttext` | Set "About" section text |
| `/setuserpic` | Upload a profile picture |
| `/setjoingroups` | Allow/disallow group invites |
| `/setprivacy` | Control message visibility in groups |

---

## Privacy Mode

By default, bots in groups only receive:
- Messages starting with `/`
- Replies to the bot's messages
- Messages where the bot is @mentioned

To receive **all messages** in a group:

1. Send `/setprivacy` to @BotFather
2. Select your bot
3. Choose **Disable**

**Note:** After changing privacy mode, you must remove and re-add the bot to affected groups for the change to take effect.

Alternatively, grant your bot **admin status** in the group.

---

## Adding to StarkBot

### Via Dashboard

1. Navigate to **Channels** in the StarkBot dashboard
2. Click **Add Channel**
3. Select **Telegram**
4. Enter your bot token
5. Save and start the channel

### Configuration Format

```json
{
  "platform": "telegram",
  "name": "My Telegram Bot",
  "config": {
    "bot_token": "123456789:ABCdefGHI..."
  }
}
```

### Environment Variable

You can also set the token via environment variable:

```bash
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHI...
```

---

## Testing Your Bot

1. Start the Telegram channel in the dashboard
2. Open Telegram and find your bot by username
3. Send `/start` or any message
4. Verify the response appears in both Telegram and the dashboard logs

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Bot not responding | Check if the channel is running in dashboard |
| "Unauthorized" error | Verify bot token is correct |
| Not receiving group messages | Check privacy mode or make bot admin |
| Token compromised | Revoke via `/revoke` in BotFather |

---

## Resources

- [Telegram Bot API Documentation](https://core.telegram.org/bots/api)
- [@BotFather](https://t.me/BotFather)
- [Bot Privacy Mode](https://core.telegram.org/bots#privacy-mode)
