# OSS 版设置页双语化 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 WarpOss 引入 rust-i18n 双语框架，完成设置窗口中英双语切换，运行时切换无需重启。

**Architecture:** 新建 `crates/warp_i18n` crate 封装 rust-i18n（YAML 语言文件编译期嵌入），新增 `general.language` 设置（auto/English/中文）驱动 `set_language()`，设置页各文本点改用 `t!()` 取词；宏设置项 description 用约定式派生 key 翻译，零宏改动。

**Tech Stack:** Rust（edition 2024）、rust-i18n 3.x、sys-locale 0.3、serde_yaml（dev）、warpui（按需渲染，仿 AppearanceEvent 模式触发重渲染）

**Spec:** `specs/oss-i18n-phase1/DESIGN.md`

## Global Constraints

- 构建环境（本机沙箱，所有 cargo 命令必须带）：
  ```bash
  export CARGO_HOME="$PWD/.cargo-home"
  export PATH="$CARGO_HOME/bin:$PATH"
  export GIT_CONFIG_NOSYSTEM=1
  export GIT_CONFIG_GLOBAL="$PWD/.gitconfig-proxy"
  export PROTOC="$PWD/.tools/protoc"
  ```
  注意：Xcode + Metal 组件已装好，**不要**再设 `WARP_LOCAL_SKIP_METAL`。
- 工具链：Homebrew rust 1.96（仓库 rust-toolchain.toml 钉 1.92，但本机无 rustup，Homebrew cargo 忽略该钉版）。
- 代码风格遵循 `AGENTS.md`：注释从简（只写 why）、避免多余类型标注、format 宏内联参数、函数上下文参数命名 `ctx` 且放最后。
- 所有用户可见字符串必须同时存在于 `locales/en.yml` 与 `locales/zh-CN.yml`（Task 1 的单测强制对称，缺 key 构建期不报错但测试失败）。
- `t!` 返回类型以实现时探针为准（String 或 Cow）；需要 `String` 的调用点统一 `t!(..).to_string()` 或 `format!`。
- 官方渠道（stable/dev/preview）行为零变化——所有改动对非 OSS 渠道保持英文原样（locale 默认 en）。
- 每个 Task 结束一个 commit，message 形如 `feat(i18n): ...`。

## Review Focus

- **中文缺失 key 必须回落英文原文**，不能显示 key 本身或 panic——Task 1 的 fallback 测试钉住。
- **运行时切换语言后，已打开窗口的所有文本立即刷新**，无残留英文（warpui 按需渲染，必须触发全窗口重渲染）——Task 2/3 的截图验证钉住。
- **search_terms 中英并存**：中文界面下输入英文关键词仍能命中对应设置项——Task 3/5 的搜索验证钉住。
- **`settings.desc.<SettingName>` 派生 key 无碰撞**，且查不到时回落宏里的英文字面量——Task 7 的单测钉住。
- **Auto 模式跟随系统 locale**：系统为英语时显示英语，为中文时显示中文，切换系统语言后重启生效——Task 2 单测 + 手动验证钉住。
- **OSS 隐藏登录功能不得影响官方渠道**：`ChannelState::channel() == Channel::Oss` 才隐藏——Task 8 的 oss 渠道构建 + stable 渠道构建检查钉住。

---

### Task 1: warp_i18n crate 骨架

**Files:**
- Create: `crates/warp_i18n/Cargo.toml`
- Create: `crates/warp_i18n/src/lib.rs`
- Create: `crates/warp_i18n/locales/en.yml`
- Create: `crates/warp_i18n/locales/zh-CN.yml`
- Create: `crates/warp_i18n/src/lib_tests.rs`
- Modify: `app/Cargo.toml`（加 `warp_i18n.workspace = true`）

**Interfaces:**
- Consumes: 无（首个任务）
- Produces:
  - `warp_i18n::set_language(language: Language) -> ()`
  - `warp_i18n::detect_system_language() -> Language`
  - `warp_i18n::t`（rust-i18n `t!` 宏再导出）
  - `warp_i18n::setting_description(setting_name: &str, fallback: &str) -> String`
  - `pub enum Language { Auto, English, Chinese }`，含 `fn locale_str(&self) -> &'static str`

- [ ] **Step 1: API 探针——确认 rust-i18n 3.x 的确切 API**

创建 crate 后先写最小 `lib.rs`（`load_locales!` + 一个 `t!` 调用），跑 `cargo check -p warp_i18n`，打印 `t!("probe")` 的类型（加 `let _: () = t!("probe");` 看编译错误推断，或直接用 `std::any::type_name_of_val`）。记录：`t!` 返回 String 还是 Cow、`set_locale` 的确切签名、`load_locales!` 是否读取 `CARGO_MANIFEST_DIR/locales`。

- [ ] **Step 2: 写失败的单测**

`crates/warp_i18n/src/lib_tests.rs`：

```rust
#[test]
fn locales_have_symmetric_keys() {
    // 解析两个 YAML，断言 zh-CN 的每个叶子 key 都存在于 en 中
    let en: serde_yaml::Value = serde_yaml::from_str(include_str!("../locales/en.yml")).unwrap();
    let zh: serde_yaml::Value = serde_yaml::from_str(include_str!("../locales/zh-CN.yml")).unwrap();
    let mut en_keys = Vec::new();
    collect_leaf_keys(&en, String::new(), &mut en_keys);
    for key in collect_leaf_keys(&zh, String::new(), &mut Vec::new()) {
        assert!(en_keys.contains(&key), "zh-CN key {key} missing in en.yml");
    }
}

#[test]
fn missing_key_falls_back_to_english() {
    warp_i18n::set_language(Language::Chinese);
    // en.yml 有 common.cancel: "Cancel"，zh-CN.yml 暂无该 key
    assert_eq!(t!("common.cancel"), "Cancel");
}

#[test]
fn setting_description_falls_back_to_literal() {
    // 无 settings.desc.FakeSetting key 时回落 fallback 参数
    assert_eq!(
        warp_i18n::setting_description("FakeSetting", "English literal"),
        "English literal"
    );
}

#[test]
fn detect_system_language_maps_chinese() {
    // 直接单测 locale_str 映射，不依赖真实系统 locale
    assert_eq!(Language::Chinese.locale_str(), "zh-CN");
    assert_eq!(Language::English.locale_str(), "en");
    assert_eq!(Language::Auto.locale_str(), "en"); // Auto 的 locale 由解析后决定
}
```

（`collect_leaf_keys` 为测试辅助 fn：递归展开 YAML mapping，产出点分层级 key 列表。）

- [ ] **Step 3: 运行测试确认失败**

```bash
cargo test -p warp_i18n 2>&1 | tail -5
```
Expected: FAIL（crate 不存在 / 函数未定义）

- [ ] **Step 4: 实现 crate**

`Cargo.toml`：`rust-i18n = "3"`、`sys-locale = "0.3"`，dev-dependencies 加 `serde_yaml`；package name `warp_i18n`，`publish.workspace = true`，`license.workspace = true`。

`lib.rs` 要点：
- `i18n::load_locales!()`（locales 目录随 crate）
- `set_language`：Auto → 调 `detect_system_language()` 解析；English/Chinese → 直接 `i18n::set_locale(locale_str)`；同时 `i18n::set_fallback("en")` 在 init 时设一次
- `detect_system_language()`：`sys_locale::get_locale()`，以 `zh` 开头 → Chinese，否则 English
- `setting_description(name, fallback)`：`let key = format!("settings.desc.{name}"); let v = t!(&key); if v == key { fallback.to_string() } else { v.to_string() }`
- `pub use rust_i18n::t;`

`locales/en.yml` 初始只放探针 key：`common: { cancel: "Cancel" }`（Step 2 的测试依赖它）。`locales/zh-CN.yml` 初始为 `common: {}`（空 mapping，让 fallback 测试成立）。

`app/Cargo.toml` dependencies 加 `warp_i18n.workspace = true`；根 `Cargo.toml` workspace.dependencies 加 `warp_i18n = { path = "crates/warp_i18n" }`（`crates/*` 已是成员，只需依赖声明）。

- [ ] **Step 5: 运行测试确认通过 + 编译 app**

```bash
cargo test -p warp_i18n 2>&1 | tail -5
cargo check -p warp --bin warp-oss 2>&1 | tail -3
```
Expected: 4 个测试 PASS；app check 通过（未使用依赖告警可接受）。

- [ ] **Step 6: Commit**

```bash
git add crates/warp_i18n app/Cargo.toml Cargo.toml Cargo.lock
git commit -m "feat(i18n): add warp_i18n crate with rust-i18n and locale files"
```

---

### Task 2: Language 设置与运行时切换

**Files:**
- Create: `app/src/settings/language.rs`
- Modify: `app/src/settings/mod.rs`（`mod language; pub use language::*;`）
- Modify: `app/src/lib.rs`（启动时按设置初始化 locale；约 1070 行 `run_internal` 内，logging init 之后）
- Modify: `app/src/appearance.rs` 或等价处（找 AppearanceEvent 触发重渲染的既有模式，新增 LanguageChangedEvent 走同一路径）
- Create: `app/src/settings/language_tests.rs`

**Interfaces:**
- Consumes: Task 1 的 `Language`/`set_language`/`detect_system_language`
- Produces:
  - `settings::Language`（重导出或类型别名，GUI 设置页用）、`settings::GeneralLanguage`（Setting 实现，`toml_path: "general.language"`）
  - `LanguageChangedEvent`（全局 action，切换后 dispatch）

- [ ] **Step 1: 写失败的单测**

`app/src/settings/language_tests.rs`：

```rust
#[test]
fn general_language_defaults_to_auto() {
    let setting = GeneralLanguage::new(None);
    assert_eq!(*setting.value(), Language::Auto);
}

#[test]
fn general_language_toml_path() {
    assert_eq!(GeneralLanguage::toml_path(), Some("general.language"));
}

#[test]
fn language_serializes_to_snake_case() {
    // 设置文件里写作 auto / english / chinese
    assert_eq!(serde_json::to_string(&Language::Auto).unwrap(), "\"auto\"");
    assert_eq!(serde_json::to_string(&Language::Chinese).unwrap(), "\"chinese\"");
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test -p warp --bin warp-oss general_language 2>&1 | tail -5
```
Expected: FAIL（模块不存在）

- [ ] **Step 3: 实现设置与初始化**

`app/src/settings/language.rs`：仿 `app/src/workspace/tab_settings.rs` 的 `define_setting!` 用法定义 `Language`（`#[derive(Default)]` Auto 为首项、serde、schemars、Copy、Clone、PartialEq、Eq）+ `GeneralLanguage`（`type: Language, default: Language::Auto, supported_platforms: ALL, group: GeneralSettings（或既有 general 组，按 tab_settings 里 new_tab_placement 所在组）, sync_to_cloud: Never, surface: GUI, private: false, toml_path: "general.language", description: "Interface language."`）。

`app/src/lib.rs`：logging init 后、UI 启动前，读 `GeneralLanguage` 当前值 → `warp_i18n::set_language(value)`。（读设置值的具体 API 仿 lib.rs 里其它启动期设置读取，如 onboarding 相关代码。）

- [ ] **Step 4: 接线运行时切换**

- `GeneralLanguage` 注册 ChangedEvent 处理：值变化 → `warp_i18n::set_language(new_value)` → dispatch `LanguageChangedEvent`
- `LanguageChangedEvent` 的处理仿 `AppearanceEvent`/`Appearance` 变更后刷新所有打开窗口的路径（在 app 的全局 action 分发处注册；如无现成全局事件机制，则在处理 fn 里对每个 `WindowId` 触发一次重渲染/invalidate，具体 API 按 appearance 主题切换的代码临摹）

- [ ] **Step 5: 运行测试 + 手动验证切换**

```bash
cargo test -p warp --bin warp-oss language 2>&1 | tail -5
```
Expected: PASS。手动：`./script/run` 启动后改 settings.toml 的 `general.language = "chinese"` 重启，确认启动即为中文框架（此时仅探针 key 有翻译，大部分仍英文，属预期）。

- [ ] **Step 6: Commit**

```bash
git add app/src/settings/language.rs app/src/settings/language_tests.rs app/src/settings/mod.rs app/src/lib.rs
git commit -m "feat(i18n): add general.language setting with runtime switching"
```

---

### Task 3: L0 窗口框架汉化 + Language 下拉

**Files:**
- Modify: `crates/warp_i18n/locales/en.yml`、`zh-CN.yml`（`settings.nav.*`、`settings.window.*`、`common.*`）
- Modify: `app/src/settings_view/mod.rs`（`SettingsSection` Display impl:345-380、umbrella 标签 1411-1429、"Search" placeholder 1360）
- Modify: `app/src/settings_view/settings_page.rs:2152`（`fn search_terms(&self) -> &str` → `-> String`，同步改调用处）
- Modify: `app/src/settings_view/main_page.rs`（Cancel/Save/Log out/Open settings file/REFERRAL_CTA 等窗口 chrome）
- Modify: `app/src/settings_view/appearance_page.rs`（新增 Language 下拉 widget）

**Interfaces:**
- Consumes: Task 1/2 的 `t!`/`GeneralLanguage`
- Produces: `search_terms() -> String` 新签名（Task 5/6 的所有 impl 依赖）

- [ ] **Step 1: 填充 L0 语言 key**

en.yml 增加（值即原英文）：
```yaml
settings:
  nav:
    about: "About"
    account: "Account"
    appearance: "Appearance"
    features: "Features"
    keybindings: "Keyboard shortcuts"
    privacy: "Privacy"
    # …其余 22 个分类 + umbrella:
  window:
    search_placeholder: "Search"
    log_out: "Log out"
    open_settings_file: "Open settings file"
    referral_cta: "Earn rewards by sharing Warp with friends & colleagues"
common:
  cancel: "Cancel"
  save: "Save"
  done: "Done"
  back: "Back"
```
zh-CN.yml 同结构中文值（"关于/账户/外观/功能/键盘快捷键/隐私…"，umbrella: agents "代理" code "代码" cloud_platform "云平台"，search_placeholder "搜索"，log_out "退出登录"，open_settings_file "打开设置文件"，referral_cta "分享 Warp 给朋友和同事，赢取奖励"，cancel "取消" save "保存" done "完成" back "返回"）。

- [ ] **Step 2: 改 `search_terms` 签名为 `String`**

`settings_page.rs:2152`：`fn search_terms(&self) -> &str;` → `fn search_terms(&self) -> String;`；把该 trait 方法的所有调用点（搜索过滤匹配处）从 `&str` 比较改为对 `String` 操作（`.as_str()` 或 contains）。机械改动，逐个 impl 之后在 Task 5/6 统一包 `t!`——本步只改签名与调用处，impl 暂返回 `"...".to_string()` 保持原值。

- [ ] **Step 3: 窗口框架替换为 `t!`**

- `SettingsSection` Display：22 个 `write!(f, "…")` → `write!(f, "{}", t!("settings.nav.<slug>"))`（slug 用该分支既有英文串的 snake_case，如 `BillingAndUsage => t!("settings.nav.billing_and_usage")`）
- umbrella 三处字面量 → `t!("settings.nav.umbrella.agents")` 等
- `set_placeholder_text("Search", ctx)` → `set_placeholder_text(t!("settings.window.search_placeholder"), ctx)`
- main_page.rs 的 chrome 字符串（含 `LOG_OUT_TEXT`/`REFERRAL_CTA` const 改为 `t!` 调用点或 const 初始化处取当前 locale）→ 对应 `settings.window.*` key
- 各 `"Cancel"`/`"Save"`/`"Done"`/`"Back"` 按钮 → `t!("common.*")`

- [ ] **Step 4: 设置页加 Language 下拉（自举）**

appearance_page.rs 新增一个 `SettingsWidget` impl（仿既有下拉 widget，如 `WindowBackdropWidget`）：label `t!("settings.appearance.language.label")`（en "Language" / zh "语言"），选项 Auto/English/中文 用 `t!("settings.appearance.language.option.auto")` 等，选中写 `GeneralLanguage`。`search_terms` 返回中英并存：`t!("settings.appearance.language.search")`（zh 值 "语言 中文 英文 language"）。

- [ ] **Step 5: 构建 + 截图验证**

```bash
./script/run --dont-open 2>&1 | tail -3
```
然后启动 app：系统语言中文时启动应直接中文；在 Language 下拉切 English/中文，截图确认侧边栏/搜索框/按钮/umbrella 全部即时切换、无残留。把截图存入 `.tmp/i18n-l0-zh.png` / `.tmp/i18n-l0-en.png`。

- [ ] **Step 6: Commit**

```bash
git add crates/warp_i18n/locales app/src/settings_view/
git commit -m "feat(i18n): localize settings window chrome and add language dropdown"
```

---

### Task 4: L1 侧边栏剩余部分与页面标题

**Files:**
- Modify: `crates/warp_i18n/locales/*.yml`（`settings.page.*`）
- Modify: `app/src/settings_view/*.rs`（8 处 `PageTitle::new(..)` + 各页 `should_render`/页头标题字面量）

**Interfaces:**
- Consumes: Task 3 的 `t!` 模式与 key 规范
- Produces: `settings.page.<page_slug>` key 全集

- [ ] **Step 1: 列页面标题清单并写 key**

`grep -rn 'PageTitle::new(' app/src/settings_view/` 取全部页标题；en.yml 加 `settings.page.<page_slug>: "<原标题>"`，zh-CN.yml 同 key 中文（Scripting 脚本、Keyboard shortcuts 键盘快捷键、Indexing and projects 索引与项目、Editor and Code Review 编辑器与代码审查、Environments 环境、API keys API 密钥、Warp Agent 代理、Profiles 配置集、MCP servers MCP 服务器、Knowledge 知识库、Third party CLI agents 第三方 CLI 代理…）。

- [ ] **Step 2: 替换调用点**

`PageTitle::new("Scripting")` → `PageTitle::new(t!("settings.page.scripting"))`（按签名实际类型加 `.to_string()`）。

- [ ] **Step 3: 页内 section 标题**

各页 render 里的 section 小标题字面量（如 Appearance 页 "Text"/"Panes"/"Blocks" 等分组头）按 `settings.<page>.section.<slug>` 补 key 并替换。

- [ ] **Step 4: 构建 + 截图验证**

`./script/run --dont-open` 后逐页截图（29 页抽查至少 10 页），确认标题中英切换正确。

- [ ] **Step 5: Commit**

```bash
git add crates/warp_i18n/locales app/src/settings_view/
git commit -m "feat(i18n): localize settings page titles and section headers"
```

---

### Task 5: L2 Appearance 页控件汉化（示范页）

**Files:**
- Modify: `crates/warp_i18n/locales/*.yml`
- Modify: `app/src/settings_view/appearance_page.rs`

**Interfaces:**
- Consumes: Task 3 的 `search_terms() -> String`
- Produces: 单页改造的完整范例（后续页照此 pattern）

- [ ] **Step 1: 遍历 appearance_page.rs 的所有用户可见字面量**

`grep -n '"[A-Z]' app/src/settings_view/appearance_page.rs` 人工过一遍，分成：widget label / widget 副标题或说明 / 下拉选项 / search_terms / 按钮。

- [ ] **Step 2: 写 key 与替换**

按 `settings.appearance.<widget_slug>.label|.description|.option.<x>|.search` 补 en/zh 两个 YAML；代码里 `render_body_item_label("Theme".to_string(), …)` → `render_body_item_label(t!("settings.appearance.theme.label"), …)`（按参数类型加 `.to_string()`）。`search_terms` impl 返回 `t!("settings.appearance.<widget>.search")`，zh 值中英并存。

- [ ] **Step 3: 构建 + 截图验证**

启动后 Appearance 页中英各截一张；在设置搜索框输入"主题"和"theme"分别确认都高亮 Theme 相关 widget。

- [ ] **Step 4: Commit**

```bash
git add crates/warp_i18n/locales app/src/settings_view/appearance_page.rs
git commit -m "feat(i18n): localize appearance settings page"
```

---

### Task 6: L2 其余页面汉化

**Files:**
- Modify: `crates/warp_i18n/locales/*.yml`
- Modify: `app/src/settings_view/main_page.rs`、`features_page.rs`、`keybindings.rs`、`privacy_page.rs`、`scripting_page.rs`、`about_page.rs`、`show_blocks_view.rs`、`code_indexing_page.rs`、`code_editor_review_page.rs`、`warp_agent_page.rs`、`agent_profiles_page.rs`、`mcp_servers_page.rs`、`knowledge_page.rs`、`cli_agents_page.rs`、`environments_page.rs`、`teams_page.rs`、`referrals_page.rs`、`warpify_page.rs`、`warp_drive_page.rs`、`features_page.rs`、`platform_page.rs`（按页存在与否调整）

**Interfaces:**
- Consumes: Task 5 的 pattern
- Produces: 全部设置页双语

- [ ] **Step 1: 按依赖序逐页改造，每页独立 commit**

顺序：main_page（账号/同步区）→ features_page → keybindings → privacy_page → scripting_page → about_page → show_blocks_view → code_indexing_page → code_editor_review_page → 其余。每页重复 Task 5 的 Step 1-3（列字面量 → 写 key → 替换 → 截图验证），单独 commit：`feat(i18n): localize <page> settings page`。

- [ ] **Step 2: keybindings 页特例**

该页有命令名/描述列表（Warp 命令词汇），属产品术语：命令名保持英文不译，描述文案纳入翻译；如工作量大，该页可在本 Task 内只译 chrome 与分类标题，命令描述列入后续阶段（在 commit message 注明）。

- [ ] **Step 3: 全页抽查**

启动后把 29 页过一遍，中英各截图抽查 10 页，记录任何遗漏字面量并补救。

- [ ] **Step 4: Commit**（每页已单独提交，本步做收尾修正的提交）

---

### Task 7: 宏设置 description 派生 key 机制

**Files:**
- Modify: `crates/warp_i18n/locales/*.yml`（`settings.desc.*`）
- Modify: `app/src/settings_view/agent_profiles_page.rs`（`code_diff_setting.description()` 等约 10 处）
- Modify: `crates/warp_i18n/src/lib_tests.rs`（补派生 key 命中用例）

**Interfaces:**
- Consumes: Task 1 的 `setting_description`
- Produces: 宏设置 description 的中文显示路径

- [ ] **Step 1: 写失败的单测**

```rust
#[test]
fn setting_description_uses_translated_value() {
    // en.yml 里放 settings.desc.ProbeSetting: "Probe description"
    warp_i18n::set_language(Language::Chinese);
    // zh-CN.yml 里放 settings.desc.ProbeSetting: "探测设置说明"
    assert_eq!(
        warp_i18n::setting_description("ProbeSetting", "Probe description"),
        "探测设置说明"
    );
}
```
（两个 YAML 加该 key；测试末尾 `set_language(Language::English)` 复位，避免污染其它测试。）

- [ ] **Step 2: 运行确认失败 → 实现**

`cargo test -p warp_i18n setting_description` 先 FAIL（zh 缺 key 时返回 fallback）。确认 fallback 逻辑正确后，本任务实际改动在 app 侧：`x.description()` → `warp_i18n::setting_description(x.setting_name(), x.description())`（agent_profiles_page.rs 里 grep `.description()` 的全部设置类调用点）。

- [ ] **Step 3: 补 `settings.desc.*` 翻译**

对 73 个带 description 的设置，按 `settings.desc.<SettingName>` 补中文（值参考原英文 description 翻译）。可分批：先 agent_profiles 页用到的，再全量。

- [ ] **Step 4: 构建 + 截图验证**

启动后进 Agent Profiles 页，确认权限类设置（code diff / read files / execute commands / write to pty）副标题为中文；切英文恢复。

- [ ] **Step 5: Commit**

```bash
git add crates/warp_i18n app/src/settings_view/agent_profiles_page.rs
git commit -m "feat(i18n): translate macro setting descriptions via derived keys"
```

---

### Task 8: OSS 渠道隐藏需登录功能

**Files:**
- Modify: `app/src/settings_view/mod.rs`（侧边栏 section 过滤处）
- Modify: 各页 `should_render`（Agents umbrella 子页、Warp Drive、Billing、Teams、Referrals 等）
- Create: `app/src/settings_view/oss_gating.rs`（集中判定 `fn is_oss_hidden(section: SettingsSection) -> bool`）

**Interfaces:**
- Consumes: 既有 `ChannelState::channel()` / `Channel::Oss`
- Produces: OSS 侧边栏与页面过滤

- [ ] **Step 1: 写失败的单测**

```rust
#[test]
fn oss_hides_login_gated_sections() {
    // 在 test-util/mock channel 状态下断言：
    assert!(is_oss_hidden(SettingsSection::WarpDrive));
    assert!(is_oss_hidden(SettingsSection::BillingAndUsage));
    assert!(is_oss_hidden(SettingsSection::Teams));
    assert!(is_oss_hidden(SettingsSection::Referrals));
    assert!(is_oss_hidden(SettingsSection::WarpAgent));
    assert!(is_oss_hidden(SettingsSection::AgentProfiles));
    assert!(!is_oss_hidden(SettingsSection::Appearance));
    assert!(!is_oss_hidden(SettingsSection::Keybindings));
}
```
（channel 状态的测试注入方式仿 `ChannelState` 既有测试基建；若无注入能力，则本测试改为纯函数测试：`is_oss_hidden` 内部读 channel，测试用 `Channel::Oss` 构造的参数化版本 `is_hidden_for_channel(section, channel)`。）

- [ ] **Step 2: 运行确认失败 → 实现**

`oss_gating.rs`：`pub(crate) fn is_hidden_for_channel(section: SettingsSection, channel: Channel) -> bool`——`channel == Channel::Oss` 时对 §8 清单里的 section 返回 true；`is_oss_hidden(section)` 读 `ChannelState::channel()` 转调。在侧边栏渲染与 `should_render` 处接入过滤（umbrella 下全部子页隐藏时 umbrella 行也不渲染）。

- [ ] **Step 3: 验证两个渠道**

```bash
./script/run --dont-open   # OSS：确认 Agents/Warp Drive/Billing/Teams/Referrals/账号登录区不可见
```
stable 渠道行为不变通过代码审查确认（`is_hidden_for_channel` 对非 Oss 恒 false）；如有渠道构建条件再跑 `cargo check` 对应 feature。

- [ ] **Step 4: Commit**

```bash
git add app/src/settings_view/
git commit -m "feat(i18n): hide login-gated settings sections in OSS channel"
```

---

### Task 9: 全量验证与 key 对称 CI 测试

**Files:**
- Modify: `crates/warp_i18n/src/lib_tests.rs`（确认对称测试覆盖最终 key 集）
- Modify: `.github/workflows/ci.yml` 或等效（如允许，加一步 `cargo test -p warp_i18n` 的显式 job；若 CI 改动超范围则跳过，仅在 PR 描述注明）

**Interfaces:**
- Consumes: Task 1-8 全部产物
- Produces: 可回归的双语验证

- [ ] **Step 1: 跑全量测试**

```bash
cargo test -p warp_i18n 2>&1 | tail -5
cargo nextest run -p warp --bin warp-oss 2>&1 | tail -5
```
Expected: 全 PASS（含 key 对称）。

- [ ] **Step 2: 缺失 key 扫描**

```bash
# 临时把系统语言切中文不现实，改为脚本扫描：代码里 t! 的 key 是否都在 en.yml
grep -rhoE 't!\("([a-z0-9_.]+)"\)' app/src | sort -u > /tmp/used_keys.txt
# 与 en.yml 叶子 key 对比，列出缺失
```
（实现方式由执行者定；缺失的补 key。）Expected: 无缺失。

- [ ] **Step 3: 端到端截图**

启动 app，系统语言中文 → 设置页全中文；Language 切 English → 全英文；搜索框分别输"主题"/"theme"命中同一 widget；截图存档 `.tmp/i18n-final-{zh,en}.png`。

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test(i18n): verify settings localization end to end"
```
