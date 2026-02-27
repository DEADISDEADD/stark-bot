A Twitter account you're watching just posted a new tweet.

**Account**: @{username}
**Tweet**: {tweet_text}
**URL**: {tweet_url}
**Posted at**: {created_at}

Your goals:
{goals}

Decide how to respond. Options:
1. Quote-tweet with commentary using `twitter_post(text="...", quote_tweet_id="{tweet_id}")`
2. Reply using `twitter_post(text="...", reply_to="{tweet_id}")`
3. Post an original tweet inspired by this using `twitter_post(text="...")`
4. Skip if not relevant

Call `task_fully_completed(summary="...")` when done.
