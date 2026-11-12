# My Axum Starter

[![CI](https://github.com/yenharvey/my-axum-starter/actions/workflows/ci.yml/badge.svg)](https://github.com/yenharvey/my-axum-starter/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.94+](https://img.shields.io/badge/rust-1.94%2B-orange.svg)](https://www.rust-lang.org)
[![Axum 0.8](https://img.shields.io/badge/axum-0.8-purple.svg)](https://github.com/tokio-rs/axum)

我的个人 Axum Web 应用模板，用于快速启动新项目。如果对你也有帮助的话就太好了。

技术栈：Axum + SeaORM + PostgreSQL，可选 Redis。

## 目录

- [特性](#特性)
- [环境要求](#环境要求)
- [快速开始](#快速开始)
- [配置](#配置)
- [项目结构](#项目结构)
- [API](#api)
- [开发](#开发)
- [部署](#部署)
- [许可证](#许可证)

## 特性

- **认证**：JWT 访问令牌 + Argon2 密码哈希，认证中间件注入当前用户
- **统一响应**：遵循 Google JSON Style Guide，成功返回 `data`，失败返回 `error`
- **统一错误**：业务错误、请求体解析失败、参数校验失败、限流都返回同一种 JSON 格式，带机器可读的原因码
- **请求校验**：`ApiJson` 提取器自动执行 `validator` 规则，按字段返回错误
- **分页**：页码分页与游标分页两种提取器
- **配置**：分层 TOML + `APP_<SECTION>_<KEY>` 环境变量覆盖，启动时校验，拼错的键直接报错
- **限流**：基于 IP 的令牌桶限流，全局与认证接口分别配置
- **中间件**：请求 ID、请求追踪、CORS、响应压缩、请求超时
- **日志**：控制台 + JSON 文件日志，按天或按小时轮转，自动清理旧文件
- **API 文档**：由代码生成 OpenAPI，内置 Scalar 文档页
- **数据库**：启动时自动迁移，可选的自动建库与演示种子数据
- **工程化**：CI、提交前检查、Docker 镜像与 GitHub Release 发布流程

## 环境要求

- Rust 1.94 或更高
- PostgreSQL（本地安装，或用 `docker compose up -d postgres` 启动）
- Redis（可选，不配置则不启用）

## 快速开始

### 1. 获取代码

用 [cargo-generate](https://github.com/cargo-generate/cargo-generate) 生成新项目：

```bash
cargo generate yenharvey/my-axum-starter --name my-new-project
```

或者直接克隆：

```bash
git clone https://github.com/yenharvey/my-axum-starter.git
```

### 2. 准备环境变量

```bash
cp .env.example .env
```

至少填写两项：

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/my_app_db
JWT_SECRET=至少32个字符的随机字符串
```

### 3. 运行

```bash
cargo run -p app
```

开发环境默认监听 <http://localhost:3001>，API 文档在 <http://localhost:3001/docs>。

启动时会依次完成：

1. 目标数据库不存在时自动创建（开发环境默认开启，需要账号有 CREATEDB 权限）
2. 执行所有未应用的数据库迁移
3. 开启了 `database.seed_demo` 时创建演示用户 `demo` / `demo-password`

### 用 Docker 一键启动

```bash
cp .env.example .env   # 至少填写 JWT_SECRET
docker compose up -d --build
```

`compose.yaml` 会同时启动应用、PostgreSQL 和 Redis，并把容器内的 `DATABASE_URL` / `REDIS_URL` 指向它们。容器内以生产配置运行，监听 3000 端口。

## 配置

所有配置项及默认值都在 [`config/default.toml`](config/default.toml) 里，按优先级从低到高依次覆盖：

| 来源 | 说明 |
| --- | --- |
| `config/default.toml` | 默认值，编译进二进制 |
| `config/{APP_ENV}.toml` | 环境专属配置，`APP_ENV` 默认为 `development` |
| `config/local.toml` | 个人本地覆盖，不进版本库 |
| `DATABASE_URL` / `REDIS_URL` / `JWT_SECRET` | 敏感信息的环境变量 |
| `APP_<SECTION>_<KEY>` | 覆盖任意配置项 |

环境变量命名规则：配置项 `section.key` 对应 `APP_<SECTION>_<KEY>`，列表用逗号分隔。

```bash
APP_SERVER_PORT=8080                    # server.port
APP_DATABASE_MAX_CONNECTIONS=20         # database.max_connections
APP_LOGGING_LEVEL=debug                 # logging.level
APP_CORS_ALLOW_ORIGINS=https://a.com,https://b.com
```

配置在启动时校验，以下情况会直接报错退出：

- 缺少必需项（`DATABASE_URL`、`JWT_SECRET`）或取值无效
- 配置文件里出现未知的键或配置段
- 出现未知的 `APP_*` 环境变量（`APP_` 前缀是本应用专用的，只有 `APP_ENV` 例外）

配置段一览：

| 配置段 | 内容 |
| --- | --- |
| `[server]` | 监听地址、端口、请求超时、是否开启 API 文档 |
| `[database]` | 连接 URL、连接池、自动建库、演示种子数据 |
| `[redis]` | 连接 URL（可选） |
| `[auth]` | JWT 密钥、令牌有效期 |
| `[logging]` | 级别、控制台格式、文件日志与轮转 |
| `[cors]` | 允许的源、方法、请求头、凭证 |

## 项目结构

```text
├── app/                          # 主应用程序
│   ├── src/
│   │   ├── main.rs               # 入口：加载配置、初始化日志、启动服务器
│   │   ├── lib.rs                # 模块声明与常用类型导出
│   │   ├── infra/                # 基础设施（与业务无关）
│   │   │   ├── config/           # 配置加载与校验（每个配置段一个文件）
│   │   │   ├── middleware/       # 认证、请求 ID、CORS、限流
│   │   │   ├── bootstrap/        # 启动引导：自动建库、演示种子数据
│   │   │   ├── logging.rs        # 日志初始化
│   │   │   ├── state.rs          # 应用状态（数据库、Redis、JWT）
│   │   │   └── server.rs         # 服务器启动、全局中间件装配、优雅关闭
│   │   ├── shared/               # 跨业务模块共享
│   │   │   ├── error/            # 错误类型（统一转换为 API 响应）
│   │   │   ├── response/         # API 响应格式
│   │   │   ├── pagination/       # 页码分页与游标分页
│   │   │   ├── json.rs           # ApiJson：带校验的 JSON 请求体提取器
│   │   │   ├── jwt.rs            # JWT 服务
│   │   │   ├── password.rs       # 密码哈希（Argon2）
│   │   │   └── from_state.rs     # 从应用状态构造服务
│   │   ├── modules/              # 业务模块（每个业务域一个目录）
│   │   │   └── user/
│   │   │       ├── mod.rs        # 路由定义
│   │   │       ├── handler.rs    # 请求处理器
│   │   │       ├── service.rs    # 业务逻辑
│   │   │       └── dto.rs        # 请求与响应的数据结构
│   │   └── routes/               # 路由装配
│   │       ├── mod.rs            # 根路由
│   │       ├── v1.rs             # V1 版本 API
│   │       ├── system.rs         # 健康检查、欢迎页、favicon
│   │       ├── docs.rs           # OpenAPI 文档
│   │       └── fallback.rs       # 404 兜底
│   ├── assets/                   # 静态文件
│   ├── templates/                # Askama 模板
│   └── tests/                    # 集成测试
├── entity/                       # 数据库实体（SeaORM）
├── migration/                    # 数据库迁移
├── config/                       # 配置文件
├── compose.yaml                  # Docker Compose 配置
├── Dockerfile
└── .env.example                  # 环境变量示例
```

### 新增业务模块

1. 在 `app/src/modules/` 下建目录，按 `dto.rs` / `handler.rs` / `service.rs` / `mod.rs` 分层，`mod.rs` 暴露 `routes()`。
2. 在 `app/src/modules/mod.rs` 里声明模块。
3. 在 `app/src/routes/v1.rs` 里挂载它的 `routes()`。

可以直接参照 `modules/user/`。

## API

### 接口一览

| 方法 | 路径 | 说明 | 认证 |
| --- | --- | --- | --- |
| GET | `/health` | 健康检查 | 否 |
| GET | `/docs` | API 文档（`server.enable_docs` 开启时） | 否 |
| POST | `/v1/user/register` | 注册 | 否 |
| POST | `/v1/user/login` | 登录，返回 JWT | 否 |
| GET | `/v1/user/me` | 当前用户信息 | 是 |
| GET | `/v1/user` | 用户列表（页码分页） | 是 |
| GET | `/v1/user/cursor` | 用户列表（游标分页） | 是 |

需要认证的接口通过 `Authorization: Bearer <token>` 传递登录返回的令牌。

限流按客户端 IP 计：全局每秒 10 个请求（突发 20），注册和登录每秒 2 个请求（突发 3）。超限返回 429。

### 示例

注册：

```bash
curl -X POST http://127.0.0.1:3001/v1/user/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123",
    "password_confirm": "password123"
  }'
```

```json
{
  "api_version": "1.0",
  "data": {
    "id": 1,
    "username": "testuser",
    "email": "test@example.com"
  }
}
```

登录：

```bash
curl -X POST http://127.0.0.1:3001/v1/user/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "testuser",
    "password": "password123"
  }'
```

```json
{
  "api_version": "1.0",
  "data": {
    "id": 1,
    "username": "testuser",
    "email": "test@example.com",
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "expires_in": 604800
  }
}
```

获取当前用户：

```bash
curl http://127.0.0.1:3001/v1/user/me \
  -H "Authorization: Bearer <token>"
```

### 分页

提供两种分页，按场景选择：

| | 页码分页 | 游标分页 |
| --- | --- | --- |
| 请求参数 | `page`（默认 1）、`page_size`（默认 20，最大 100） | `limit`（默认 20，最大 100）、`cursor` |
| 跳到任意页、显示总数 | 支持 | 不支持 |
| 翻到很深的页 | 越深越慢 | 速度恒定 |
| 翻页期间有新数据 | 可能重复或遗漏 | 不受影响 |
| 适合 | 后台管理表格 | 信息流、无限滚动、大表 |

页码分页：

```bash
curl "http://127.0.0.1:3001/v1/user?page=1&page_size=20" \
  -H "Authorization: Bearer <token>"
```

```json
{
  "api_version": "1.0",
  "data": {
    "kind": "UserList",
    "items": [
      { "id": 2, "username": "bob", "email": "bob@example.com", "status": "active" },
      { "id": 1, "username": "alice", "email": "alice@example.com", "status": "active" }
    ],
    "current_item_count": 2,
    "items_per_page": 20,
    "start_index": 0,
    "total_items": 2,
    "page_index": 1,
    "total_pages": 1
  }
}
```

游标分页：

```bash
curl "http://127.0.0.1:3001/v1/user/cursor?limit=20" \
  -H "Authorization: Bearer <token>"
```

```json
{
  "api_version": "1.0",
  "data": {
    "kind": "UserList",
    "items": [
      { "id": 2, "username": "bob", "email": "bob@example.com", "status": "active" }
    ],
    "current_item_count": 1,
    "next_cursor": "Mg"
  }
}
```

把响应里的 `next_cursor` 作为下一次请求的 `cursor` 参数即可取下一页；响应里没有 `next_cursor` 表示已经到底。

在处理器里使用分页，只需要把提取器写进参数：

```rust
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    pagination: Pagination, // 或 CursorPagination
) -> Result<ApiResponse<UserListItem>, AppError> {
    let (items, total) = UserService::from_state(&state).list_users(pagination).await?;
    Ok(ApiResponse::page(items, total, pagination).with_kind("UserList"))
}
```

### 错误格式

所有错误响应的结构相同。`code` 是 HTTP 状态码，`errors[].reason` 是机器可读的原因码，可用作前端的 i18n key；字段校验错误带 `location`。

```json
{
  "api_version": "1.0",
  "error": {
    "code": 400,
    "message": "email: 邮箱格式无效; password: 密码长度至少 8 个字符",
    "errors": [
      {
        "domain": "validation",
        "reason": "INVALID_EMAIL",
        "message": "邮箱格式无效",
        "location": "email",
        "location_type": "body"
      },
      {
        "domain": "validation",
        "reason": "INVALID_LENGTH",
        "message": "密码长度至少 8 个字符",
        "location": "password",
        "location_type": "body"
      }
    ]
  }
}
```

服务器内部错误（数据库、Redis 等）只返回 `Internal server error`，细节只写入日志。

请求体校验规则写在请求结构体上，处理器用 `ApiJson` 提取：

```rust
#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreatePost {
    #[validate(length(min = 1, max = 100, message = "标题长度必须在 1-100 个字符之间"))]
    pub title: String,
}

pub async fn create_post(ApiJson(req): ApiJson<CreatePost>) -> Result<ApiResponse<Post>, AppError> {
    // 到这里 req 已经通过校验
}
```

## 开发

### 常用命令

```bash
make check       # 格式化 + clippy
make test        # 运行全部测试
make changelog   # 用 git-cliff 生成 CHANGELOG.md
```

首次运行 `cargo test` 时会安装提交前钩子（cargo-husky），之后每次提交前自动检查格式和 clippy。

### 测试

```bash
cargo test --workspace
```

- 单元测试与路由测试在各模块的 `tests` 子模块里，不需要数据库
- 集成测试在 `app/tests/`

### 数据库迁移

应用启动时会自动执行迁移。需要手动操作时使用 [sea-orm-cli](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)：

```bash
sea-orm-cli migrate generate create_xxx_table   # 创建新的迁移文件
sea-orm-cli migrate up                          # 应用未执行的迁移
sea-orm-cli migrate down                        # 回滚最后一个迁移
```

### 生成实体

```bash
sea-orm-cli generate entity -o entity/src --lib
```

注意：生成器会覆盖 `entity/src/` 下的同名文件。`user.rs` 的 `status` 字段手工改成了 `UserStatus` 枚举，重新生成后需要把这类调整改回来。

## 部署

### Docker 镜像

```bash
docker build -t my-app --target runtime-app .
```

运行时至少需要提供 `DATABASE_URL` 和 `JWT_SECRET`。镜像默认 `APP_ENV=production`，监听 3000 端口。

生产环境建议：

- 前端与 API 不同源时，设置 `APP_CORS_ALLOW_ORIGINS`（默认不允许任何跨域请求）
- 部署在反向代理之后时，限流需要改用 `SmartIpKeyExtractor` 读取真实客户端 IP，见 `infra/middleware/rate_limit.rs`
- 不要开启 `database.seed_demo`，演示用户的密码是公开的

### 发布

推送 `v*` 标签会触发发布流程：

```bash
git tag v0.1.0
git push origin v0.1.0
```

流程会用 git-cliff 生成更新说明并创建 GitHub Release，同时构建镜像推送到 `ghcr.io/<owner>/my-axum-starter`。

## 许可证

[MIT](LICENSE)
