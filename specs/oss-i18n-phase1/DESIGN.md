# OSS 版双语化设计文档（Phase 1：设置页面）

> 状态：待评审
> 范围：WarpOss 构建的设置窗口（Settings）中英双语，运行时切换
> 关联需求：界面跟随系统语言，或在设置里手动切中/英，同一时刻只显示一种

## 1. 背景与目标

OSS 版（WarpOss）当前零 i18n 基础设施：无 i18n crate、无语言检测、无翻译文件，
全部 UI 文字为硬编码英文字面量（仅 `app/src/settings_view/` 就有约 1,400 个
用户可见字符串，183 个 SettingsWidget，186 个 search_terms）。

本阶段目标：

1. 引入 rust-i18n + YAML 语言文件，搭建可运行时切换的双语框架
2. 新增语言设置 `general.language`（auto 跟随系统 / English / 中文），切换无需重启
3. 完成**设置页面**的完整汉化（窗口框架 + 侧边栏 + 各设置页 + 搜索）
4. 中文缺失的 key 自动回退英文，支持增量翻译

非目标（后续阶段）：主界面（终端、blocks、命令面板、onboarding）汉化；
Warp 官方渠道（stable/dev）的翻译。

## 2. 范围与优先级

设置窗口的翻译面按优先级分三层，Phase 1 内按序完成：

| 层 | 内容 | 数量 | 优先级 |
|---|---|---|---|
| L0 窗口框架 | 侧边栏分类、umbrella 标签、搜索框、通用按钮、账号区 | ~40 | P0 |
| L1 页面标题与导航 | 29 个设置页的 PageTitle、页内 section 标题 | ~35 | P0 |
| L2 控件文本 | 183 个 widget 的 label/副标题/search_terms、73 个宏设置 description | ~1,400 | P1 |

## 3. 总体架构

```
crates/warp_i18n/           # 新 crate，封装 rust-i18n
├── Cargo.toml
├── src/lib.rs              # set_language() / t! 再导出 / locale 检测
└── locales/
    ├── en.yml              # 英文（源语言，key → 英文原串）
    └── zh-CN.yml           # 中文翻译
```

- 依赖 `rust-i18n = "3"`，YAML 语言文件通过 `i18n::load_locales!()` **编译期嵌入**
  二进制，.app bundle 无需外部文件，运行时切换零 IO
- `warp_i18n` 提供三个公开 API：
  - `set_language(Language)` — 切换语言（写入全局 locale）
  - `detect_system_language() -> Language` — 读系统 locale（`sys-locale` crate）
  - `t!` — 直接再导出 rust-i18n 的 `t!` 宏，业务代码 `use warp_i18n::t;`
- 回退：rust-i18n 原生支持，中文缺 key 时回落英文

## 4. 语言设置

- 新增 `app/src/settings/language.rs`：
  ```rust
  pub enum Language { #[default] Auto, English, Chinese }
  // toml_path: "general.language"，SyncToCloud::Never，surface: GUI
  ```
- 启动时：`Language::Auto` → `detect_system_language()`（zh* → Chinese，其余 English）
- 切换时（设置页改动即生效）：
  1. `GeneralLanguage` 的 ChangedEvent 触发 `warp_i18n::set_language()`
  2. 发 `LanguageChangedEvent`，仿照主题切换（AppearanceEvent）的既有模式
     触发所有打开窗口重渲染——warpui 按需渲染，locale 变更后下一帧自然取新文案
- 设置页自身放一个"Language / 语言"下拉（放在 Appearance 页顶部或 Account 页），
  用本框架的 `t!` 渲染选项文本，作为自举示范

## 5. Key 命名规范

点分层级，全小写 snake_case，与代码位置一一对应：

```
settings.nav.<section_slug>          侧边栏分类，如 settings.nav.appearance
settings.nav.umbrella.<slug>         umbrella 分组标签（agents/code/cloud_platform）
settings.page.<page_slug>            页面标题，如 settings.page.appearance
settings.<page_slug>.<widget_slug>.label       控件主标签
settings.<page_slug>.<widget_slug>.description 控件副标题
settings.<page_slug>.<widget_slug>.search      search_terms（中英关键词都放这里）
settings.window.<element>            窗口框架（search_placeholder / log_out / open_settings_file …）
settings.desc.<SettingName>          宏设置项的 description（见 §6）
common.<action>                      跨页通用按钮（cancel/save/done/back…）
```

- `<widget_slug>` 取 widget 结构体名的 snake_case（如 `ThemeSelectWidget` →
  `theme_select`），保证唯一且可从代码直接读出
- YAML 中英文 key 完全对称；`zh-CN.yml` 允许暂缺（回落英文）

## 6. 宏设置项的翻译机制

宏（`define_setting!`/`define_settings_group!`）里的 `description` 是
`&'static str` 字面量，同时服务两处：JSON schema 生成（开发者产物，保持英文），
以及部分设置页 UI 副标题（如 `agent_profiles_page.rs` 里的
`code_diff_setting.description()`）。

**方案：约定式派生 key，零宏改动。**

- 每个 setting 的 `setting_name()`（`stringify!($name)`）全局唯一，
  天然可作 key 后缀：`settings.desc.<SettingName>`
- `warp_i18n` 提供：
  ```rust
  pub fn setting_description<S: Setting>(setting: &S) -> String {
      let key = format!("settings.desc.{}", S::setting_name());
      let translated = t!(&key);          // 缺失时 t! 返回 key 本身
      if translated == key { setting.description().to_string() } else { translated }
  }
  ```
- 改造点：UI 里 `x.description()` → `warp_i18n::setting_description(&x)`（约 10 处调用点）
- schema 生成链路不动，继续输出英文字面量

## 7. 设置页字符串清单（Phase 1 交付物）

按 §2 分层，全部落入两个 YAML：

1. **窗口框架（L0）**
   - `SettingsSection` Display 的 22 个分类（`app/src/settings_view/mod.rs:345`）
   - umbrella 标签："Agents" / "Code" / "Cloud platform"（mod.rs:1411-1429）
   - 搜索框 placeholder："Search"（mod.rs:1360）
   - 通用按钮："Cancel"×21、"Save"×6、"Open settings file"×6、"Log out"×3、
     "Done"、"Back"
   - main_page.rs 窗口 chrome：REFERRAL_CTA、账号区标题、同步开关等
2. **页面标题（L1）**：8 个显式 `PageTitle::new(..)` + 29 个 `SettingsPageViewHandle`
   对应页的页头（含 Agent/Code/Cloud 各子页）
3. **控件文本（L2）**：183 个 widget 的 render 内字面量 + 186 个 search_terms
   + 73 个宏 description（经 §6 机制）
   - search_terms 中文化策略：`zh-CN.yml` 中该 key 的值写成
     "主题 外观 theme appearance"——中英关键词并存，两种语言搜索都能命中

## 8. 移除"需登录才能用"的功能（OSS 版）

与汉化并行推进，**策略是隐藏入口而非删代码**（删代码会破坏核心路径且无法跟随上游）：

| 功能 | 现状 | OSS 处理 |
|---|---|---|
| Agent Mode / AI 面板 | `is_any_ai_enabled=false` 默认关 | 隐藏侧边栏 Warp Agent/Profiles/MCP/Knowledge/ThirdParty CLI agents 整个 Agents umbrella |
| Warp Drive / 云同步 | SyncToCloud 设置 | 隐藏 Warp Drive 页 + 账号页同步开关 |
| Billing and usage | 需登录 | 隐藏入口 |
| Teams / Referrals | 需登录 | 隐藏入口 |
| 账号区（登录/登出/头像） | 登录墙 | OSS 下隐藏或替换为"OSS 构建"说明 |
| 会话分享、Cloud environments、API keys | 需登录 | 隐藏入口 |

实现方式：在 `should_render()`/`SettingsSection` 过滤处增加
`ChannelState::channel().is_dogfood() == false && channel == Oss` 判定
（OSS 渠道专用编译期或运行时开关，不动官方渠道行为）。

## 9. 风险与缓解

| 风险 | 缓解 |
|---|---|
| 1,400 字符串一次改完，PR 巨大易冲突 | 分 L0/L1/L2 三个提交；L2 按页拆分（Appearance 先行） |
| 运行时切换后部分缓存文本不刷新 | 仿主题切换的 LanguageChangedEvent 全窗口重渲染；截图逐页验证 |
| search_terms 只译中文导致英文搜不到 | zh-CN 值中英并存（§7.3） |
| rust-i18n 与 warpui 文本渲染（FormattedText）集成 | label 均为 `String`/`&str` 入参，`t!` 返回 Cow<str>，直接适配 |
| 上游更新冲突 | 改动全部集中在 locales/ + warp_i18n/ + settings_view 的 t! 包裹，冲突面小且机械 |

## 10. 实施顺序（供 writing-plans 展开）

1. `crates/warp_i18n` 骨架 + en.yml 空壳 + 依赖接入 app crate
2. `Language` 设置 + 系统检测 + 运行时切换 + 重渲染接线
3. L0 窗口框架汉化（含设置页 Language 下拉，自举验证）
4. L1 页面标题与侧边栏汉化
5. L2 控件文本汉化，按页推进：Appearance → Account/main → Features →
   Keybindings → Privacy → 其余（每页一个可独立验证的提交）
6. OSS 登录功能隐藏（§8）
7. 全量截图验证 + 缺失 key 检查（CI 可加 yaml key 对称性测试）

## 11. 验证方式

- `./script/run` 构建 WarpOss，启动后逐页截图，核对中英切换
- 系统语言切中文后重启，验证 auto 模式生效
- 设置搜索框输入中/英关键词，验证均可命中
- `cargo nextest`：warp_i18n 单测（key 对称性、回退、setting_description 回退逻辑）
