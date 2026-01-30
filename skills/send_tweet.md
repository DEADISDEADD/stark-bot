---
name: send_tweet
description: "Send a tweet to Twitter/X. Use exec tool with curl to post."
version: 1.0.0
author: starkbot
homepage: https://developer.x.com/en/docs/twitter-api/tweets/manage-tweets/api-reference/post-tweets
metadata: {"requires_auth": true}
requires_tools: [exec]
requires_binaries: [curl]
tags: [twitter, x, tweet, post, social-media]
---

# Send Tweet

Post a tweet to Twitter/X using the exec tool with curl.

## Requirements

- `TWITTER_TOKEN` must be set in API Keys settings
- The token must be an OAuth 2.0 User Access Token with `tweet.write` scope

## Post a Tweet

Use the exec tool to run:

```bash
curl -s -X POST "https://api.twitter.com/2/tweets" \
  -H "Authorization: Bearer $TWITTER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"text": "YOUR_TWEET_TEXT_HERE"}'
```

## Examples

### Simple tweet
```bash
curl -s -X POST "https://api.twitter.com/2/tweets" \
  -H "Authorization: Bearer $TWITTER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world from StarkBot!"}'
```

### Tweet with special characters (use jq for safe JSON encoding)
```bash
TEXT="Line 1
Line 2 with emoji: fire"
curl -s -X POST "https://api.twitter.com/2/tweets" \
  -H "Authorization: Bearer $TWITTER_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$(jq -n --arg t "$TEXT" '{text: $t}')"
```

### Reply to a tweet
```bash
curl -s -X POST "https://api.twitter.com/2/tweets" \
  -H "Authorization: Bearer $TWITTER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"text": "My reply!", "reply": {"in_reply_to_tweet_id": "TWEET_ID_HERE"}}'
```

### Quote tweet
```bash
curl -s -X POST "https://api.twitter.com/2/tweets" \
  -H "Authorization: Bearer $TWITTER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"text": "Check this out!", "quote_tweet_id": "TWEET_ID_HERE"}'
```

## Success Response

```json
{
  "data": {
    "id": "1234567890123456789",
    "text": "Hello world from StarkBot!"
  }
}
```

The tweet URL will be: `https://twitter.com/i/status/{id}`

## Error Handling

| Code | Meaning | Solution |
|------|---------|----------|
| 401 | Unauthorized | Check TWITTER_TOKEN in API Keys |
| 403 | Forbidden | Token lacks tweet.write scope |
| 429 | Rate Limited | Wait 15 minutes |

## Character Limit

Tweets are limited to 280 characters. The API will reject tweets that exceed this limit.
