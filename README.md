# 🚀 tinyredis

> A lightweight Redis reimplementation in Rust — fast, embeddable, and easy to understand.

![Rust](https://img.shields.io/badge/Rust-💛-orange)
![License](https://img.shields.io/github/license/wangyi-kai/tinyredis)
![Status](https://img.shields.io/badge/status-WIP-red)

---

## ✨ 项目简介

**tinyredis** 是一个用 Rust 编写的 Redis 重实现，旨在学习 Redis 内部机制，并构建一个高性能、简洁易读的内存键值数据库。它兼容 RESP 协议，支持基础数据结构，并具备异步网络 IO 和高效内存管理。

---

## 🧱 Features

* 🧠 **学习友好**：核心数据结构和命令解析逻辑简洁明了
* ⚡ **异步运行**：基于 `tokio` 的异步网络模型
* 🧵 **多数据库支持**：兼容 Redis 的多 DB 架构
* 💾 **RESP 协议解析**：支持 Redis 原生协议通信
* 🔧 **模块化设计**：便于扩展指令与数据结构
* 🧪 **测试覆盖**：包含单元测试和基准测试

---

## 🚀 快速开始

### 构建项目

```bash
git clone https://github.com/wangyi-kai/tinyredis.git
cd tinyredis/src
```

### 启动 tinyredis

```
bash启动服务端
cargo run --release --bin redis_server
bash启动客户端
cargo run --release --bin redis_cli

默认监听地址为 `127.0.0.1:8000`

---

## 📦 已支持命令

* `HSET key field value`
* `HGET key field`
* `HDEL key field`
* 更多命令持续开发中...

---

## 🧪 测试

*待开发

---

## 📚 架构设计

```
src/
├── server/      # 异步网络服务
├── protocol/    # RESP 协议解析器
├── db/          # 数据库与数据结构实现
├── command/     # 命令分发与执行逻辑
└── main.rs
```

---

## 🛠️ 技术栈

* [Rust](https://www.rust-lang.org/)
* [Tokio](https://tokio.rs/)
* [Bytes](https://docs.rs/bytes)
* [Serde](https://serde.rs/)
* [Tracing](https://docs.rs/tracing)
* [Clap](https://docs.rs/clap)

---

## 📈 未来计划

* [ ] 支持 RDB / AOF 持久化
* [ ] 发布 Docker 镜像
* [ ] 发布 benchmark 工具
* [ ] 实现事务（MULTI/EXEC）
* [ ] Lua 脚本支持
* [ ] 集群协议兼容

---

## ❤️ 致谢

* [Redis](https://redis.io/)
* [mini-redis](https://github.com/tokio-rs/mini-redis)
* [sled](https://github.com/spacejam/sled)

---

## 📄 License

[MIT](LICENSE)

---

## 🗨️ 联系我

欢迎在 [GitHub Discussions](https://github.com/wangyi-kai/tinyredis/discussions) 提问交流，或提交 Issue / PR 🙌
