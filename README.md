# My Axum Starter

我的个人 Axum Web 应用模板，用于快速启动新项目。如果对你也有帮助的话就太好了。

## 特性

- **Web框架**: Axum + SeaORM + PostgreSQL
- **认证系统**: JWT令牌认证 + Argon2密码哈希
- **限流控制**: tower_governor IP限流（全局10req/s，认证端2req/s）
- **配置管理**: 分层 TOML 配置 + `APP_<SECTION>_<KEY>` 环境变量覆盖，启动时校验
- **错误处理**: 统一的错误处理，HTTP状态码+错误原因映射
- **API响应**: 遵循 Google JSON Style Guide 标准，支持分页、资源元数据
- **分页**: 统一的 `page` / `page_size` 查询参数与校验，附用户列表示例接口
- **数据库引导**: 可选的自动建库与演示种子数据
- **文档生成**: OpenAPI 文档（`server.enable_docs` 开关，开发环境默认开启）
- **中间件栈**: 日志追踪、CORS、压缩、请求ID、请求超时等
- **日志系统**: 控制台 + JSON 文件日志，支持轮转和自动清理旧文件

## 项目结构

```shell
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
│   │   │   ├── response/         # API 响应格式（Google JSON Style Guide）
│   │   │   ├── jwt.rs            # JWT 服务
│   │   │   ├── password.rs       # 密码哈希（Argon2）
│   │   │   ├── pagination.rs     # 分页查询参数
│   │   │   └── from_state.rs     # 从应用状态构造服务
│   │   ├── modules/              # 业务模块（每个业务域一个目录）
│   │   │   └── user/
│   │   │       ├── mod.rs        # 路由定义
│   │   │       ├── handler.rs    # 请求处理器
│   │   │       ├── service.rs    # 业务逻辑
│   │   │       └── dto.rs        # 数据传输对象
│   │   └── routes/               # 路由装配
│   │       ├── mod.rs            # 根路由
│   │       ├── v1.rs             # V1 版本 API
│   │       ├── system.rs         # 健康检查、欢迎页、favicon
│   │       ├── docs.rs           # OpenAPI 文档
│   │       └── fallback.rs       # 404 兜底
│   ├── assets/                   # 静态文件
│   ├── templates/                # Askama 模板
│   ├── tests/                    # 集成测试
│   └── Cargo.toml
├── entity/                       # 数据库实体（SeaORM）
├── migration/                    # 数据库迁移
├── config/
│   ├── default.toml              # 所有配置项及默认值（唯一来源）
│   ├── development.toml          # 开发环境覆盖
│   └── production.toml           # 生产环境覆盖
├── compose.yaml                  # Docker Compose 配置
├── Dockerfile
├── .env.example                  # 环境变量示例
├── Cargo.toml                    # Workspace 配置
└── README.md
```

新增业务模块时：在 `modules/` 下建目录（`dto` / `handler` / `service` / `mod.rs`），
然后在 `routes/v1.rs` 里挂载它的 `routes()`。

## 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/yenharvey/my-axum-starter.git
cd my-axum-starter
```

OR

```shell
cargo generate yenharvey/my-axum-starter

cargo generate yenharvey/my-axum-starter --name my-new-project
```

### 2. 环境配置

```bash
cp .env.example .env
```

然后至少填写两个必需项：

```bash
DATABASE_URL=postgresql://postgres:your_password@localhost:5432/your_database
JWT_SECRET=至少32个字符的随机字符串
```

### 3. 数据库设置

开发环境下（`database.auto_create = true`）目标数据库不存在时会自动创建，前提是当前账号能连接 `postgres` 管理库并拥有 CREATEDB 权限。
需要一个现成的账号来试接口时，设置 `APP_DATABASE_SEED_DEMO=true` 会创建演示用户 `demo` / `demo-password`。

应用启动时会自动执行迁移，无需手动操作。需要单独执行时：

```bash
sea-orm-cli migrate up
```

### 4. 运行

```bash
cargo run -p app
```

访问 http://localhost:3001

## 配置

所有配置项及默认值都在 [`config/default.toml`](config/default.toml) 里，按优先级从低到高依次覆盖：

| 来源 | 说明 |
| --- | --- |
| `config/default.toml` | 默认值（编译进二进制） |
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

配置在启动时校验，缺少必需项或取值无效会直接报错退出。

### Docker

```bash
cp .env.example .env   # 至少填写 JWT_SECRET
docker compose up -d --build
```

`compose.yaml` 会同时启动 PostgreSQL 和 Redis，并把容器内的 `DATABASE_URL` / `REDIS_URL` 指向它们。
容器内默认 `APP_ENV=production`，监听 3000 端口。

只想在本机用 `cargo run` 开发时，可以只启动依赖：

```bash
docker compose up -d postgres redis
```

## API 文档

`server.enable_docs = true` 时（开发环境默认开启）访问：http://localhost:3001/docs

## API 测试指南

### 1. 用户注册

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

**响应示例：**
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

### 2. 用户登录

```bash
curl -X POST http://127.0.0.1:3001/v1/user/login \
  -H "Content-Type: application/json" \
  -d '{
    "username_or_email": "testuser",
    "password": "password123"
  }'
```

**响应示例：**
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

### 3. 获取当前用户信息

使用登录返回的 token，在 Authorization header 中以 Bearer 格式传递：

```bash
curl -X GET http://127.0.0.1:3001/v1/user/me \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

**响应示例：**
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

### 4. 用户列表（分页）

```bash
curl "http://127.0.0.1:3001/v1/user?page=1&page_size=20" \\
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

`page` 从 1 开始（默认 1），`page_size` 取值 1-100（默认 20）。

### 5. 健康检查

```bash
curl http://127.0.0.1:3001/health
```

## 常用命令

```bash
# 运行应用
cargo run -p app

# 数据库迁移
sea-orm-cli migrate up

# 回滚最后一个迁移
sea-orm-cli migrate down

# 生成数据库模型实体
sea-orm-cli generate entity -o entity/src

# 创建新的迁移文件
sea-orm-cli migrate generate create_xxx_table
```

## 许可证

[MIT](LICENSE)
