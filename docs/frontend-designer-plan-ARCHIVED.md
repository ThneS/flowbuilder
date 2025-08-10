# FlowBuilder Frontend Workflow Designer 规划 (已归档)

> 归档时间: 2025-08-10
> 状态: 已完成规划，尚未实施。准备迁出到独立仓库。

## 目标
- 提供可视化方式创建 / 编辑 / 导入 / 导出 FlowBuilder 工作流 YAML，减少手写错误。
- 支持任务(Task) / 动作(Action) / 重试 / 超时 / 条件 / 变量 / 环境变量 等结构化编辑。

## 技术栈 (MVP)
- React 18 + TypeScript
- Vite 构建
- TailwindCSS UI
- Zustand (状态) + React Hook Form (表单) + yaml 序列化

## MVP 核心功能
1. 基本信息：version, env, vars
2. 任务与动作 CRUD + 排序
3. 重试 / 超时 配置表单
4. 条件 (next_if) 文本表达式输入
5. YAML 实时预览（只读 + 复制 / 下载）
6. 导入（YAML/JSON）→ 解析 → AST
7. 校验（重复 ID / 不存在引用 / 策略参数合法性）
8. LocalStorage 自动草稿保存

## 后续阶段（摘要）
- Phase 2: DAG 视图、模板片段库、Diff 对比、Undo/Redo
- Phase 3: 后端 /dry-run 计划预览 API、远程模板、权限

## 数据模型 (核心片段)
```ts
interface WorkflowConfig { workflow: { version: string; env: Record<string,string>; vars: Record<string,string>; tasks: TaskWrapper[]; }; }
interface TaskWrapper { task: { id: string; name: string; description?: string; actions: ActionWrapper[]; }; }
interface ActionWrapper { action: { id: string; name: string; description?: string; type: 'builtin'|'cmd'|'http'|'wasm'; flow: { next?: string|null; next_if?: string; retry?: RetryConfig; timeout?: TimeoutConfig; }; outputs?: Record<string,string>; parameters?: Record<string,any>; }; }
```

## 验证规则
| 规则 | 类型 |
|------|------|
| 任务 ID 唯一 | error |
| 动作 ID 唯一 (task.action) | error |
| retry.strategy 参数匹配 | error |
| timeout.duration > 0 | error |
| flow.next 引用存在 | error |
| 悬挂节点 | warning |
| 空 next_if | warning |

## 目录结构建议
```
src/
  components/workflow/*
  state/workflowStore.ts
  utils/yaml.ts
  pages/DesignerPage.tsx
```

## 里程碑
| 阶段 | 周期 | 交付 |
|------|------|------|
| Phase 0 | 0.5w | 模型+校验规则确定 |
| Phase 1 | 2w | MVP 可用编辑器 |
| Phase 2 | 2w | DAG + 模板 + Diff |
| Phase 3 | 2w | 后端联动 + 版本化 |

## 迁出建议
单独仓库: `flowbuilder-designer`
- MIT / Apache-2.0 双许可（与主项目一致）
- CI: typecheck + lint + vitest
- 可发布 GitHub Pages 作为在线 Demo

## 后续
本文件仅归档规划。实施需在新仓库重新初始化并执行脚手架搭建。
