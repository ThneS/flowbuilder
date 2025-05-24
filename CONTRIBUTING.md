# 贡献指南

感谢你考虑为 FlowBuilder 做出贡献！我们欢迎任何形式的贡献，包括但不限于：

- 报告 bug
- 提出新功能建议
- 改进文档
- 提交代码修复
- 添加新功能
- 改进测试用例

## 开发环境设置

1. 确保你已安装 Rust 工具链：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. 克隆仓库：
   ```bash
   git clone https://github.com/ThneS/flowbuilder.git
   cd flowbuilder
   ```

3. 安装开发依赖：
   ```bash
   cargo install cargo-watch
   cargo install cargo-llvm-cov
   ```

## 开发流程

1. 创建新分支：
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-fix-name
   ```

2. 运行测试：
   ```bash
   cargo test
   cargo test --all-features
   ```

3. 检查代码格式：
   ```bash
   cargo fmt
   cargo clippy
   ```

4. 运行基准测试：
   ```bash
   cargo bench
   ```

5. 提交更改：
   ```bash
   git add .
   git commit -m "描述你的更改"
   ```

6. 推送到远程：
   ```bash
   git push origin feature/your-feature-name
   ```

7. 创建 Pull Request

## 代码风格

- 遵循 Rust 标准代码风格
- 使用 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 保持代码简洁和可读性
- 添加适当的注释和文档

## 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` 修复 bug
- `docs:` 文档更改
- `style:` 代码风格更改
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建过程或辅助工具的变动

示例：
```
feat: 添加并行执行支持
fix: 修复上下文快照恢复问题
docs: 更新 API 文档
```

## 测试规范

- 为所有新功能添加单元测试
- 为公共 API 添加集成测试
- 确保测试覆盖率达到 80% 以上
- 运行 `cargo llvm-cov` 检查测试覆盖率

## 文档规范

- 所有公共 API 必须有文档注释
- 使用示例代码说明用法
- 保持文档与代码同步更新
- 文档使用 Markdown 格式

## Pull Request 流程

1. 确保你的 PR 描述清晰说明更改内容
2. 确保所有测试通过
3. 确保代码符合风格指南
4. 更新相关文档
5. 等待代码审查

## 发布流程

1. 更新 `Cargo.toml` 中的版本号
2. 更新 `CHANGELOG.md`
3. 创建发布标签
4. 发布到 crates.io

## 行为准则

请参阅 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) 了解我们的行为准则。

## 问题反馈

- 使用 GitHub Issues 报告问题
- 提供详细的复现步骤
- 包含环境信息
- 如果可能，提供最小复现示例

## 功能请求

- 使用 GitHub Issues 提出功能请求
- 清晰描述功能用途
- 说明可能的实现方案
- 讨论潜在的影响

## 许可证

通过贡献代码，你同意你的贡献将使用与项目相同的许可证（Apache License 2.0）。

## 联系方式

- 项目维护者：ThneS
- GitHub Issues：https://github.com/ThneS/flowbuilder/issues
- 邮件：your-email@example.com

感谢你的贡献！