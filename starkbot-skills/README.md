# StarkBot Skills

This directory contains skill definitions for StarkBot. Skills provide integrations with external services and APIs.

## Available Skills

| Skill | Description | Tags |
|-------|-------------|------|
| [bankr](bankr.md) | Interact with Bankr - check token info, wallet balances, and use the Agent API | crypto, defi, wallet, yield |
| [discord](discord.md) | Discord integration for messaging and webhooks | social-media, messaging |
| [github](github.md) | GitHub integration for repository operations | git, development |
| [local_wallet](local_wallet.md) | Local wallet management | crypto, wallet |
| [openssl](openssl.md) | OpenSSL utilities for cryptographic operations | security, crypto |
| [scheduling](scheduling.md) | Task scheduling capabilities | automation |
| [stock-analysis](stock-analysis.md) | Stock market analysis tools | finance, trading |
| [twitter](twitter.md) | Post tweets and interact with Twitter/X | social-media, posting |
| [weather](weather.md) | Weather information and forecasts | weather, utilities |

## Skill Structure

Each skill is defined in a markdown file with:

1. **YAML Frontmatter** - Metadata including:
   - `name`: Skill identifier
   - `description`: Brief description
   - `version`: Semantic version
   - `author`: Skill author
   - `homepage`: Reference documentation URL
   - `metadata`: Additional config (auth requirements, emoji, binary dependencies)
   - `requires_binaries`: System binaries needed (e.g., curl, jq)
   - `tags`: Searchable tags

2. **Documentation** - Markdown content with:
   - Setup instructions
   - API endpoints and examples
   - Authentication details
   - Usage examples

## Adding a New Skill

1. Create a new `.md` file in this directory
2. Add the YAML frontmatter with required fields
3. Document the skill's capabilities and usage
4. Add example commands and API calls
