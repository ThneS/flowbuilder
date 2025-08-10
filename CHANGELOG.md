# 更新日志

本文档记录项目的所有重要变更。

## [未发布]

### 新增

-   （预留）

### 变更

-   （预留）

### 修复

-   （预留）

### 移除

-   （预留）

## [0.1.1] - 2025-08-10

### 新增

-   runtime 子特性：`parallel` / `retry` / `perf-metrics` / `detailed-logging` 精细化拆分
-   基于 feature 组合的 CI 构建矩阵（最小 / 并行重试 / yaml-only / full-debug 等）
-   性能基准 (Criterion) `executor_features` 对比串行与并行执行
-   YAML crate 增加 `perf-metrics` 透传特性

### 变更

-   YAML crate 不再隐式 re-export runtime 类型，明确 feature 边界
-   facade prelude 去除 runtime 通配符导出，改为显式列出，消除歧义 re-export 警告
-   执行器/编排器内部使用 cfg(feature) 移除运行期开关 bool 字段

### 修复

-   关闭 runtime 或 perf-metrics 时的编译错误与未定义引用
-   CI workflow 表达式语法错误修正

### 移除

-   旧的运行时布尔配置开关（改为 feature）

### 新增

-   初始化项目结构
-   添加核心功能实现
    -   FlowBuilder 构建器
    -   上下文管理
    -   步骤定义
    -   错误处理
-   添加示例代码
    -   基础用法示例
    -   子流程示例
    -   并行执行示例
-   添加文档
    -   API 参考文档
    -   高级用法指南
    -   快速入门指南
-   添加 GitHub 配置
    -   CI/CD 工作流
    -   Issue 和 PR 模板
    -   安全策略
    -   行为准则
    -   贡献指南
-   添加开发工具配置
    -   Rustfmt 配置
    -   Clippy 配置
    -   EditorConfig 配置

### 变更

-   更新 LICENSE 文件，添加版权信息

### 修复

-   无

### 移除

-   无

## [0.1.0] - 2024-03-XX

### 新增

-   项目初始化
-   核心功能实现：
    -   基本步骤执行
    -   命名步骤
    -   条件步骤
    -   错误处理步骤
    -   等待逻辑
    -   上下文共享
-   基础文档
-   单元测试
-   CI/CD 配置

### 变更

-   无

### 修复

-   无

### 移除

-   无

## 版本说明

### 版本号格式

-   主版本号：当你做了不兼容的 API 修改
-   次版本号：当你做了向下兼容的功能性新增
-   修订号：当你做了向下兼容的问题修正

### 版本发布周期

-   主版本：重大更新，可能包含破坏性变更
-   次版本：新功能发布，向下兼容
-   修订版本：问题修复，向下兼容

### 更新日志格式

每个版本的更新日志包含以下部分：

-   新增：新功能
-   变更：现有功能的变更
-   修复：问题修复
-   移除：废弃功能的移除

### 贡献指南

-   所有重要的更改都应该记录在此文件中
-   使用清晰的分类和描述
-   保持更新日志的简洁性
-   使用过去时态描述更改

## 链接

-   [GitHub 仓库](https://github.com/ThneS/flowbuilder)
-   [crates.io](https://crates.io/crates/flowbuilder)
-   [文档](https://docs.rs/flowbuilder)

## 致谢

感谢所有为这个项目做出贡献的开发者。
