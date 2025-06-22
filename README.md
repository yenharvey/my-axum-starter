# My Axum Starter

我的个人 Axum Web 应用模板，用于快速启动新项目。如果对你也有帮助的话就太好了。

## 特性

- Axum + SeaORM + PostgreSQL
- 支持配置文件和环境变量
- 统一的错误处理和 API 响应格式
- OpenAPI 文档生成（debug 模式）
- 基础中间件（日志、CORS、限流等）

## 项目结构

```shell
├── app/                     # 主应用程序
│   ├── src/
│   │   ├── main.rs
│   │   ├── core/            # 核心功能
│   │   │   ├── config.rs    # 配置管理
│   │   │   ├── logging.rs   # 日志
│   │   │   ├── response.rs  # API响应
│   │   │   ├── state.rs     # 应用状态
│   │   │   └── middleware/
│   │   ├── error/           # 错误处理
│   │   ├── modules/         # 业务模块
│   │   │   ├── auth/        # 认证模块
│   │   │   └── docs/        # 文档
│   │   ├── routes/          # 路由
│   │   └── shared/          # 共享工具
│   ├── assets/              # 静态文件
│   └── templates/           # HTML模板
├── entity/                  # 数据库实体
├── migration/               # 数据库迁移
└── config.toml              # 配置文件
```

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

创建 `.env` 文件：

```bash
# 必需
DATABASE_URL=postgresql://postgres:your_password@localhost:5432/your_database
JWT_SECRET=your-jwt-secret-key

# 可选
REDIS_URL=redis://localhost:6379
APP_SERVER_HOST=127.0.0.1
APP_SERVER_PORT=3001
APP_LOGGING_LEVEL=info
```

### 3. 数据库设置

```bash
# 运行迁移
cargo run -p migration
```

### 4. 运行

```bash
cargo run -p app
```

访问 http://localhost:3001

## 配置

### config.toml

```toml
[server]
host = "127.0.0.1"
port = 3001
timeout = 30

[database]
max_connections = 10
pool_timeout = 30

[logging]
level = "info"
format = "pretty"
```

## API 文档

debug 模式下访问：http://localhost:3001/docs

## 常用命令

```bash
# 运行应用
cargo run -p app

# 数据库迁移
cargo run -p migration

# 数据库模型实体生成
sea-orm-cli generate entity -o entity/src
```

## 许可证

[MIT](LICENSE)
