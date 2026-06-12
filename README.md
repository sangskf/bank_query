# 金融机构查询系统

基于 Rust + Warp 的金融机构（银行）查询系统，支持模糊搜索、Excel 导入导出、数据管理等操作。

## 快速开始

```bash
# 启动服务器（默认端口 10119）
cargo run

# 指定端口
PORT=8080 cargo run
```

打开浏览器访问 `http://localhost:10119`。

## 配置文件

系统使用 `config.toml` 作为配置文件，首次启动时自动创建，默认内容如下：

```toml
admin_password = "admin123"
```

可通过环境变量 `CONFIG_PATH` 指定自定义配置文件路径：

```bash
CONFIG_PATH=/etc/bank_query/config.toml cargo run
```

通过命令行修改密码：

```bash
cargo run set-password your_new_password
```

## 命令行用法

### 启动服务器（默认）

```bash
cargo run
# 或
cargo run server
```

### 初始化数据库

```bash
cargo run init-db
```

创建数据库文件 `bank_query.db` 和默认配置文件 `config.toml`。

### 设置管理员密码

```bash
cargo run set-password <新密码>
```

更新 `config.toml` 中的管理员密码。

### 从 Excel 导入数据

```bash
cargo run import <文件路径.xlsx>
```

直接通过命令行导入 Excel 文件到数据库，无需启动 Web 服务器。

### 清空所有数据

```bash
cargo run clear
```

清空数据库中所有记录。

## Web 界面

### 搜索

在搜索框输入联行号或金融机构名称，实时模糊搜索，匹配内容红色高亮显示。

### 操作密码

以下操作需要输入管理员密码（通过 `config.toml` 配置）：

- **导入数据** — 上传 Excel 文件
- **清空数据** — 删除全部记录
- **编辑** — 修改单条记录的联行号和名称
- **删除** — 删除单条记录

### Excel 模板

点击「下载模板」获取标准导入模板（含联行号、金融机构名称两列）。

### 页面特性

- 实时搜索（200ms 防抖）
- 复制联行号 / 复制名称
- 响应式设计，适配移动端
- 背景动态模糊色块动画

## API 接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 首页 |
| GET | `/api/banks?q=关键词` | 模糊搜索 |
| POST | `/api/banks/import` | 上传 Excel 导入 |
| DELETE | `/api/banks` | 清空全部数据 |
| PUT | `/api/banks/{id}` | 更新单条记录 |
| DELETE | `/api/banks/{id}` | 删除单条记录 |
| GET | `/api/template/download` | 下载 Excel 模板 |
| POST | `/api/verify-password` | 验证管理员密码 |

## 技术栈

- **Web 框架**: Warp 0.3
- **数据库**: SQLite（rusqlite + r2d2 连接池）
- **Excel**: rust_xlsxwriter（写入）、calamine（读取）
- **前端**: 原生 HTML/CSS/JavaScript（无框架）

## 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 代码检查
cargo clippy

# 格式化
cargo fmt

# 测试
cargo test
```

## CI/CD

推送 `v*` 标签到 GitHub 时自动构建 Windows 版本：

```bash
git tag v1.0.0
git push origin v1.0.0
```
