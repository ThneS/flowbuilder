# FlowBuilder Frontend Workflow Designer 规划 (已归档)

> 归档时间: 2025-08-10
> 状态: 已完成规划，尚未实施。准备迁出到独立仓库。

## 目标

-   提供可视化方式创建 / 编辑 / 导入 / 导出 FlowBuilder 工作流 YAML，减少手写错误。
-   支持任务(Task) / 动作(Action) / 重试 / 超时 / 条件 / 变量 / 环境变量 等结构化编辑。

## 技术栈 (MVP)

-   React 18 + TypeScript
-   Vite 构建
-   TailwindCSS UI
-   Zustand (状态) + React Hook Form (表单) + yaml 序列化

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

-   Phase 2: DAG 视图、模板片段库、Diff 对比、Undo/Redo
-   Phase 3: 后端 /dry-run 计划预览 API、远程模板、权限

## 数据模型 (核心片段)

```ts
interface WorkflowConfig {
    workflow: { version: string; env: Record<string, string>; vars: Record<string, string>; tasks: TaskWrapper[] };
}
interface TaskWrapper {
    task: { id: string; name: string; description?: string; actions: ActionWrapper[] };
}
interface ActionWrapper {
    action: { id: string; name: string; description?: string; type: "builtin" | "cmd" | "http" | "wasm"; flow: { next?: string | null; next_if?: string; retry?: RetryConfig; timeout?: TimeoutConfig }; outputs?: Record<string, string>; parameters?: Record<string, any> };
}
```

## 验证规则

| 规则                       | 类型    |
| -------------------------- | ------- |
| 任务 ID 唯一               | error   |
| 动作 ID 唯一 (task.action) | error   |
| retry.strategy 参数匹配    | error   |
| timeout.duration > 0       | error   |
| flow.next 引用存在         | error   |
| 悬挂节点                   | warning |
| 空 next_if                 | warning |

## 目录结构建议

```
src/
  components/workflow/*
  state/workflowStore.ts
  utils/yaml.ts
  pages/DesignerPage.tsx
```

## 里程碑

| 阶段    | 周期 | 交付              |
| ------- | ---- | ----------------- |
| Phase 0 | 0.5w | 模型+校验规则确定 |
| Phase 1 | 2w   | MVP 可用编辑器    |
| Phase 2 | 2w   | DAG + 模板 + Diff |
| Phase 3 | 2w   | 后端联动 + 版本化 |

## 更新（2025-08-12）：Dify 风格拖拽编排（静态原型）

为满足“类似 Dify 的拖拉拽进行动作编排”的需求，新增了一个零依赖、可直接在浏览器打开的静态原型页面，操作体验参考 Dify：

-   入口：docs/designer.html（在仓库中直接双击或用本地服务器打开即可）
-   组件库：Cmd / HTTP / Builtin / WASM 四类动作，可拖拽到画布
-   连线：点击源节点的 pin 后再点击目标节点，生成 flow.next 单向连线
-   多任务：支持创建多个 Task，任务间节点相互独立
-   自动排版：基于 next 链的简易布局
-   属性编辑：节点名称、描述就地编辑
-   导出/导入：
    -   导出 JSON/YAML，结构与现有 WorkflowConfig 完整对齐
    -   导入 JSON 恢复画布（支持 workflow.tasks[].task.actions[].action 结构）

数据模型映射（与 flowbuilder-yaml 一致）：

```jsonc
{
    "workflow": {
        "version": "1.0",
        "env": {},
        "vars": {},
        "template": null,
        "tasks": [
            {
                "task": {
                    "id": "task_xxx",
                    "name": "任务名称",
                    "description": "",
                    "actions": [
                        {
                            "action": {
                                "id": "act_xxx",
                                "name": "动作名称",
                                "description": "",
                                "flow": { "next": "act_yyy" },
                                "outputs": {},
                                "type": "cmd|http|builtin|wasm",
                                "parameters": {}
                            }
                        }
                    ]
                }
            }
        ]
    }
}
```

已知限制（原型阶段）：

-   仅支持单一 next，不含条件分支（next_if）、循环（while）、重试/超时、on_error/on_timeout 等高级流控
-   节点删除、连线删除、键盘操作等仍是基础能力，可在正式实现中完善
-   参数编辑器为占位，后续需要 schema 驱动的表单（包含校验）

建议的下一步（迁移到正式前端实现）：

-   在 React/TS 实现中加入：
    -   条件分支（next_if）与带标签的边；循环/重试/超时/错误处理表单
    -   参数表单（类型感知）、动作模板库、片段复用
    -   连线/节点的增删改、快捷键、对齐/对齐线、迷你 minimap
    -   校验（悬挂节点、无效引用、重复 ID）与一键修复
-   支持 YAML/JSON 的相互导入导出与 Diff
-   与后端执行计划预览（/dry-run）和模板仓库联动

## 迁出建议

单独仓库: `flowbuilder-designer`

-   MIT / Apache-2.0 双许可（与主项目一致）
-   CI: typecheck + lint + vitest
-   可发布 GitHub Pages 作为在线 Demo

## 后续

本文件仅归档规划。实施需在新仓库重新初始化并执行脚手架搭建。
