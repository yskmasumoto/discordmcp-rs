# discordmcp-rs
discord mcp server

## 設定

`config.toml` をプロジェクト直下に作成し、以下を設定してください。

```toml
[discord]
bot_token = "YOUR_BOT_TOKEN"
channel_id = "YOUR_CHANNEL_ID"
base_url = "https://discord.com/api/v10"
```

## 起動

```bash
cargo run
```

## MCPツール

### `send_message`

指定したチャンネルに文字列メッセージを送信します。

引数:

```json
{
	"content": "Hello from MCP"
}
```
