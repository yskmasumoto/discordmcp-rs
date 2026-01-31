# discordmcp-rs
discord mcp server

## 設定

`config.toml` をプロジェクト直下に作成し、以下を設定してください。

```toml
[discord]
bot_token = "YOUR_BOT_TOKEN"
channel_id = "YOUR_CHANNEL_ID"
base_url = "https://discord.com/api/v10"

[mcp]
# JSON Schema の $schema フィールドを削除する場合は true
disable_schema_url = false
```

## 起動

```bash
cargo run
```

## mcp.json 例

以下は `.vscode/mcp.json` の例です。

```json
{
	"servers": {
		"discordmcp": {
			"type": "stdio",
			"command": "cargo",
			"args": ["run", "--quiet"],
			"env": {
				"RUST_LOG": "info"
			}
		}
	}
}
```

`config.toml` はプロジェクト直下に配置してください。

## MCPツール

### `send_message`

指定したチャンネルに文字列メッセージを送信します。

引数:

```json
{
	"content": "Hello from MCP"
}
```
