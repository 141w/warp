# 免登录 pi harness：必开的设置（各会话必读）

本仓库构建的 **warp-oss** 支持不登录 Warp 账号直接跑 pi harness（CLI 无头 run 和 GUI
Agent Mode 入口）。这一切换在一个设置项后面，**默认关闭**——不打开的话免登录能力完全不生效。

## 要打开的东西（唯一一步）

编辑 `~/.warp-oss/settings.toml`（注意是 **warp-oss** 的数据目录，不是 `~/.warp`——后者是
stable 渠道的），加上：

```toml
[agents.warp_agent]
local_pi_only_enabled = true
```

已确认本机该文件于 2026-11-08 配好此项。

## 打开后得到什么

| 路径 | 效果 |
|---|---|
| CLI 无头 | `warp-oss agent run --harness pi ...` 不再要求登录（四道登录闸 + 凭据透传全部就绪） |
| GUI 入口 | 未登录时 Agent Mode 面板键位、输入消息栏、上下文菜单“附加为 agent 模式上下文”可见；harness 列表解析为仅 Pi |
| Agent conversations 面板 | 对免登录模式**保持隐藏**（面板内容是云端同步历史，本地 run 的对话不在里面）——这是刻意行为，不是 bug |

## 用法（CLI，Quorum Hub 唤醒形态）

```bash
OPENROUTER_API_KEY=<key> warp-oss agent run --harness pi \
  --model openrouter/nvidia/nemotron-3-ultra-550b-a55b:free \
  --mcp '<hub shim JSON>' -p '<提示词>'
```

要点：

- **凭据走环境变量**：`OPENROUTER_API_KEY` / `OPENAI_API_KEY` / `ANTHROPIC_API_KEY`
  （可配同名 `*_BASE_URL`）会被写进 pi 本次 run 的隔离 auth.json；warp 的 managed
  secrets（需登录）优先，环境变量只填空缺。
- **模型必须是 pi 提供的型号**（`provider/model`，免费档带 `:free` 后缀）。可用列表可用
  `pi --mode rpc` 的 `get_available_models` 探到；key 无效时列表为空、会话会被拒。
- 前置条件不变：本机 profile 的 onboarding 需已完成（或 `AgentOnboarding` 关闭），
  且 `SkipFirebaseAnonymousUser` 旗标开启（默认开）——否则未登录的根视图会进 onboarding，
  run 在终端创建处失败。

## 重要边界

- **该设置只对未登录生效**。已登录用户照旧走 `is_any_ai_enabled`（AI 总开关）逻辑，
  行为零变化；登录后 harness 列表恢复服务器下发。
- **官方 Warp app（/Applications/Warp.app）不受任何影响**——那是官方二进制，不是本仓库
  构建。要验证免登录能力必须跑本仓库的 `warp-oss`（`./script/run`）。
- 免登录 run 结束时向 server 上传 transcript/对话数据会失败（未登录没有 token），但是
  非致命（WARN，退出码仍为 0）。

## 相关提交

- `2ec5dd9a` feat(ai): finish the account-free pi harness run path（闸门 + env 凭据透传 + ACP 豁免）
- `9d5051fb` feat(ai): offer agent entry points to logged-out local pi-only users（GUI 入口）
- 更早的闸门改动随 `2fad8c21`（i18n 会话的提交）并入

E2E 隔离环境（可复现真实 key 往返）在 `/tmp/warp-e2e-home`：`HOME` 指向它、PATH 含
`~/.local/bin`（pi）、`~/.local/node24/bin`（node）、`/tmp/warp-e2e-home/.npm-global/bin`
（pi-acp）即可。
