//! # FlowBuilder Runtime - 增强的流程编排器
//!
//! 基于执行计划的流程编排器，负责生成和优化执行计划

use anyhow::Result;
use flowbuilder_core::{
    ExecutionNode, ExecutionPhase, ExecutionPlan, FlowPlanner,
    PhaseExecutionMode,
};
use std::collections::HashMap;

/// 增强的流程编排器
pub struct EnhancedFlowOrchestrator {
    /// 优化配置
    config: OrchestratorConfig,
}

/// 编排器配置
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// 是否启用并行优化
    pub enable_parallel_optimization: bool,
    /// 最大并行度
    pub max_parallelism: usize,
    /// 是否启用依赖分析
    pub enable_dependency_analysis: bool,
    /// 是否启用条件优化
    pub enable_condition_optimization: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            enable_parallel_optimization: true,
            max_parallelism: 10,
            enable_dependency_analysis: true,
            enable_condition_optimization: true,
        }
    }
}

impl Default for EnhancedFlowOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedFlowOrchestrator {
    /// 创建新的编排器
    pub fn new() -> Self {
        Self {
            config: OrchestratorConfig::default(),
        }
    }

    /// 使用配置创建编排器
    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self { config }
    }

    /// 从节点列表创建执行计划
    pub fn create_execution_plan(
        &self,
        nodes: Vec<ExecutionNode>,
        env_vars: HashMap<String, serde_yaml::Value>,
        flow_vars: HashMap<String, serde_yaml::Value>,
        workflow_name: String,
        workflow_version: String,
    ) -> Result<ExecutionPlan> {
        let mut plan = ExecutionPlan::new(
            workflow_name,
            workflow_version,
            env_vars,
            flow_vars,
        );

        // 1. 构建依赖图
        let dependency_graph = self.build_dependency_graph(&nodes)?;

        // 2. 执行拓扑排序
        let sorted_layers = self.topological_sort(&nodes, &dependency_graph)?;

        // 3. 生成执行阶段
        let phases = self.create_execution_phases(sorted_layers)?;

        // 4. 添加阶段到计划
        for phase in phases {
            plan.add_phase(phase);
        }

        // 5. 优化计划
        if self.config.enable_parallel_optimization {
            self.optimize_for_parallelism(&mut plan)?;
        }

        // 6. 验证计划
        plan.validate()
            .map_err(|e| anyhow::anyhow!("执行计划验证失败: {}", e))?;

        Ok(plan)
    }

    /// 构建依赖图
    fn build_dependency_graph(
        &self,
        nodes: &[ExecutionNode],
    ) -> Result<HashMap<String, Vec<String>>> {
        let mut graph = HashMap::new();

        for node in nodes {
            graph.insert(node.id.clone(), node.dependencies.clone());
        }

        // 验证依赖的有效性
        for (node_id, deps) in &graph {
            for dep in deps {
                if !graph.contains_key(dep) {
                    return Err(anyhow::anyhow!(
                        "节点 {} 依赖的节点 {} 不存在",
                        node_id,
                        dep
                    ));
                }
            }
        }

        Ok(graph)
    }

    /// 拓扑排序，生成执行层次
    fn topological_sort(
        &self,
        nodes: &[ExecutionNode],
        _graph: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<Vec<ExecutionNode>>> {
        let mut layers = Vec::new();
        let mut remaining_nodes: HashMap<String, ExecutionNode> =
            nodes.iter().map(|n| (n.id.clone(), n.clone())).collect();
        let mut in_degree = HashMap::new();

        // 计算入度
        for node in nodes {
            in_degree.insert(node.id.clone(), node.dependencies.len());
        }

        // 分层处理
        while !remaining_nodes.is_empty() {
            let mut current_layer = Vec::new();

            // 找出当前层可以执行的节点（入度为0）
            let ready_nodes: Vec<String> = in_degree
                .iter()
                .filter(|(_, &degree)| degree == 0)
                .map(|(id, _)| id.clone())
                .collect();

            if ready_nodes.is_empty() {
                return Err(anyhow::anyhow!("检测到循环依赖"));
            }

            // 添加到当前层
            for node_id in ready_nodes {
                if let Some(node) = remaining_nodes.remove(&node_id) {
                    current_layer.push(node);
                    in_degree.remove(&node_id);
                }
            }

            // 更新剩余节点的入度
            for node in &current_layer {
                for other_node in remaining_nodes.values() {
                    if other_node.dependencies.contains(&node.id) {
                        if let Some(degree) = in_degree.get_mut(&other_node.id)
                        {
                            *degree -= 1;
                        }
                    }
                }
            }

            layers.push(current_layer);
        }

        Ok(layers)
    }

    /// 创建执行阶段
    fn create_execution_phases(
        &self,
        layers: Vec<Vec<ExecutionNode>>,
    ) -> Result<Vec<ExecutionPhase>> {
        let mut phases = Vec::new();

        for (index, layer) in layers.into_iter().enumerate() {
            let execution_mode = if layer.len() == 1 {
                PhaseExecutionMode::Sequential
            } else if layer.len() <= self.config.max_parallelism {
                PhaseExecutionMode::Parallel
            } else {
                // 如果节点数超过最大并行度，分批处理
                PhaseExecutionMode::Parallel
            };

            let phase = ExecutionPhase {
                id: format!("phase_{index}"),
                name: format!("执行阶段 {}", index + 1),
                execution_mode,
                nodes: layer,
                condition: None,
            };

            phases.push(phase);
        }

        Ok(phases)
    }

    /// 优化并行执行
    fn optimize_for_parallelism(&self, plan: &mut ExecutionPlan) -> Result<()> {
        if !self.config.enable_parallel_optimization {
            return Ok(());
        }

        // 分析每个阶段的并行可能性
        for phase in &mut plan.phases {
            if phase.nodes.len() > self.config.max_parallelism {
                // 如果节点数量超过最大并行度，需要分批
                let chunks: Vec<Vec<ExecutionNode>> = phase
                    .nodes
                    .chunks(self.config.max_parallelism)
                    .map(|chunk| chunk.to_vec())
                    .collect();

                // 创建子阶段
                let mut sub_phases = Vec::new();
                for (i, chunk) in chunks.into_iter().enumerate() {
                    let sub_phase = ExecutionPhase {
                        id: format!("{}_sub_{}", phase.id, i),
                        name: format!("{} - 子阶段 {}", phase.name, i + 1),
                        execution_mode: PhaseExecutionMode::Parallel,
                        nodes: chunk,
                        condition: phase.condition.clone(),
                    };
                    sub_phases.push(sub_phase);
                }

                // 注意：这里需要重新构建计划结构来支持子阶段
                // 为了简化，这里只是记录优化信息
                println!(
                    "阶段 {} 被优化为 {} 个子阶段",
                    phase.name,
                    sub_phases.len()
                );
            }
        }

        Ok(())
    }

    /// 分析执行计划的复杂度
    pub fn analyze_complexity(
        &self,
        plan: &ExecutionPlan,
    ) -> ExecutionComplexity {
        let mut total_nodes = 0;
        let mut max_parallel_nodes = 0;
        let mut total_dependencies = 0;
        let mut conditional_nodes = 0;

        for phase in &plan.phases {
            total_nodes += phase.nodes.len();

            if matches!(phase.execution_mode, PhaseExecutionMode::Parallel) {
                max_parallel_nodes = max_parallel_nodes.max(phase.nodes.len());
            }

            for node in &phase.nodes {
                total_dependencies += node.dependencies.len();
                if node.condition.is_some() {
                    conditional_nodes += 1;
                }
            }
        }

        ExecutionComplexity {
            total_nodes,
            total_phases: plan.phases.len(),
            max_parallel_nodes,
            total_dependencies,
            conditional_nodes,
            complexity_score: self.calculate_complexity_score(
                total_nodes,
                total_dependencies,
                conditional_nodes,
                max_parallel_nodes,
            ),
        }
    }

    /// 计算复杂度分数
    fn calculate_complexity_score(
        &self,
        total_nodes: usize,
        total_dependencies: usize,
        conditional_nodes: usize,
        max_parallel_nodes: usize,
    ) -> f64 {
        let base_score = total_nodes as f64;
        let dependency_penalty = (total_dependencies as f64) * 0.5;
        let condition_penalty = (conditional_nodes as f64) * 0.3;
        let parallel_bonus = (max_parallel_nodes as f64) * 0.2;

        base_score + dependency_penalty + condition_penalty - parallel_bonus
    }
}

/// 执行复杂度分析结果
#[derive(Debug, Clone)]
pub struct ExecutionComplexity {
    /// 总节点数
    pub total_nodes: usize,
    /// 总阶段数
    pub total_phases: usize,
    /// 最大并行节点数
    pub max_parallel_nodes: usize,
    /// 总依赖关系数
    pub total_dependencies: usize,
    /// 条件节点数
    pub conditional_nodes: usize,
    /// 复杂度分数
    pub complexity_score: f64,
}

impl FlowPlanner for EnhancedFlowOrchestrator {
    type Input = (
        Vec<ExecutionNode>,
        HashMap<String, serde_yaml::Value>,
        HashMap<String, serde_yaml::Value>,
        String,
        String,
    );
    type Output = ExecutionPlan;
    type Error = anyhow::Error;

    fn create_execution_plan(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let (nodes, env_vars, flow_vars, workflow_name, workflow_version) =
            input;
        self.create_execution_plan(
            nodes,
            env_vars,
            flow_vars,
            workflow_name,
            workflow_version,
        )
    }

    fn optimize_plan(
        &self,
        mut plan: Self::Output,
    ) -> Result<Self::Output, Self::Error> {
        self.optimize_for_parallelism(&mut plan)?;
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowbuilder_core::{ActionSpec, ExecutionNode};

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = EnhancedFlowOrchestrator::new();
        assert!(orchestrator.config.enable_parallel_optimization);
    }

    #[test]
    fn test_dependency_graph_build() {
        let orchestrator = EnhancedFlowOrchestrator::new();

        let node1 = ExecutionNode::new(
            "node1".to_string(),
            "Node 1".to_string(),
            ActionSpec {
                action_type: "test".to_string(),
                parameters: HashMap::new(),
                outputs: HashMap::new(),
            },
        );

        let node2 = ExecutionNode::new(
            "node2".to_string(),
            "Node 2".to_string(),
            ActionSpec {
                action_type: "test".to_string(),
                parameters: HashMap::new(),
                outputs: HashMap::new(),
            },
        )
        .add_dependency("node1".to_string());

        let nodes = vec![node1, node2];
        let graph = orchestrator.build_dependency_graph(&nodes).unwrap();

        assert_eq!(graph.len(), 2);
        assert_eq!(graph.get("node1").unwrap().len(), 0);
        assert_eq!(graph.get("node2").unwrap().len(), 1);
    }
}
