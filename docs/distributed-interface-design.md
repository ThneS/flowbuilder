# FlowBuilder 分布式接口预留设计

## 设计理念

在第一阶段的单体架构中，预留完整的分布式接口，采用 NoOp/Local 实现，确保：

1. **接口稳定性**: 接口定义一次性到位，避免后续大规模重构
2. **平滑演进**: 只需替换实现，不改变接口
3. **测试就绪**: 可在单机环境中模拟分布式场景
4. **配置驱动**: 通过配置切换部署模式

## 核心接口设计

### 1. 共识层接口

#### ConsensusEngine - 共识引擎

```rust
/// 分布式共识引擎接口
/// 第一阶段: NoOp实现 (直接通过)
/// 第二阶段: Raft实现 (强一致性)
/// 第三阶段: Byzantine实现 (拜占庭容错)
#[async_trait]
pub trait ConsensusEngine: Send + Sync {
    /// 提案状态变更
    async fn propose(&self, change: StateChange) -> Result<ProposalId>;

    /// 对提案投票
    async fn vote(&self, proposal_id: ProposalId, vote: Vote) -> Result<()>;

    /// 提交已达成共识的变更
    async fn commit(&self, proposal_id: ProposalId) -> Result<()>;

    /// 查询提案状态
    async fn status(&self, proposal_id: ProposalId) -> Result<ProposalStatus>;

    /// 获取当前Leader
    async fn get_leader(&self) -> Result<Option<NodeId>>;

    /// 触发Leader选举
    async fn trigger_election(&self) -> Result<()>;
}

/// 第一阶段的NoOp实现
pub struct NoOpConsensus;

impl ConsensusEngine for NoOpConsensus {
    async fn propose(&self, _change: StateChange) -> Result<ProposalId> {
        Ok(ProposalId::immediate())
    }

    async fn vote(&self, _proposal_id: ProposalId, _vote: Vote) -> Result<()> {
        Ok(()) // 直接通过
    }

    async fn commit(&self, _proposal_id: ProposalId) -> Result<()> {
        Ok(()) // 立即提交
    }

    async fn status(&self, _proposal_id: ProposalId) -> Result<ProposalStatus> {
        Ok(ProposalStatus::Committed)
    }

    async fn get_leader(&self) -> Result<Option<NodeId>> {
        Ok(Some(NodeId::local())) // 单机模式自己是Leader
    }

    async fn trigger_election(&self) -> Result<()> {
        Ok(()) // NoOp
    }
}
```

#### NodeManager - 节点管理

```rust
/// 集群节点管理接口
#[async_trait]
pub trait NodeManager: Send + Sync {
    /// 注册节点到集群
    async fn register(&self, node_info: NodeInfo) -> Result<NodeId>;

    /// 发现集群中的其他节点
    async fn discover(&self) -> Result<Vec<NodeInfo>>;

    /// 监控节点健康状态
    async fn health_check(&self, node_id: NodeId) -> Result<NodeHealth>;

    /// 处理节点离线
    async fn handle_offline(&self, node_id: NodeId) -> Result<()>;

    /// 获取集群拓扑
    async fn topology(&self) -> Result<ClusterTopology>;

    /// 广播消息到所有节点
    async fn broadcast(&self, message: ClusterMessage) -> Result<()>;
}

/// 第一阶段的单机实现
pub struct LocalNodeManager {
    local_node: NodeInfo,
}

impl NodeManager for LocalNodeManager {
    async fn register(&self, _node_info: NodeInfo) -> Result<NodeId> {
        Ok(self.local_node.id)
    }

    async fn discover(&self) -> Result<Vec<NodeInfo>> {
        Ok(vec![self.local_node.clone()])
    }

    async fn health_check(&self, _node_id: NodeId) -> Result<NodeHealth> {
        Ok(NodeHealth::Healthy)
    }

    async fn handle_offline(&self, _node_id: NodeId) -> Result<()> {
        Ok(()) // 单机模式无需处理
    }

    async fn topology(&self) -> Result<ClusterTopology> {
        Ok(ClusterTopology::single_node(self.local_node.clone()))
    }

    async fn broadcast(&self, _message: ClusterMessage) -> Result<()> {
        Ok(()) // 单机模式无需广播
    }
}
```

### 2. 状态同步接口

#### StateSynchronizer - 状态同步器

```rust
/// 分布式状态同步接口
#[async_trait]
pub trait StateSynchronizer: Send + Sync {
    /// 同步工作流状态
    async fn sync_workflow(&self, workflow_id: WorkflowId) -> Result<WorkflowState>;

    /// 同步任务状态
    async fn sync_task(&self, task_id: TaskId) -> Result<TaskState>;

    /// 广播状态变更
    async fn broadcast_change(&self, change: StateChange) -> Result<()>;

    /// 订阅状态变更
    async fn subscribe_changes(&self) -> Result<Receiver<StateChange>>;

    /// 检查状态一致性
    async fn check_consistency(&self) -> Result<ConsistencyReport>;

    /// 强制同步所有状态
    async fn force_sync(&self) -> Result<()>;
}

/// 第一阶段的本地实现
pub struct LocalStateSynchronizer {
    state_store: Arc<dyn StateStore>,
    event_bus: Arc<dyn EventBus>,
}

impl StateSynchronizer for LocalStateSynchronizer {
    async fn sync_workflow(&self, workflow_id: WorkflowId) -> Result<WorkflowState> {
        self.state_store.get_workflow_state(workflow_id).await
    }

    async fn sync_task(&self, task_id: TaskId) -> Result<TaskState> {
        self.state_store.get_task_state(task_id).await
    }

    async fn broadcast_change(&self, change: StateChange) -> Result<()> {
        self.event_bus.publish("state_change", change).await
    }

    async fn subscribe_changes(&self) -> Result<Receiver<StateChange>> {
        self.event_bus.subscribe("state_change").await
    }

    async fn check_consistency(&self) -> Result<ConsistencyReport> {
        Ok(ConsistencyReport::consistent())
    }

    async fn force_sync(&self) -> Result<()> {
        Ok(()) // 单机模式无需同步
    }
}
```

### 3. 分布式协调接口

#### DistributedLock - 分布式锁

```rust
/// 分布式锁接口
#[async_trait]
pub trait DistributedLock: Send + Sync {
    /// 获取锁
    async fn acquire(&self, resource: &str, ttl: Duration) -> Result<LockGuard>;

    /// 尝试获取锁 (非阻塞)
    async fn try_acquire(&self, resource: &str, ttl: Duration) -> Result<Option<LockGuard>>;

    /// 续期锁
    async fn renew(&self, guard: &LockGuard, ttl: Duration) -> Result<()>;

    /// 检查锁状态
    async fn status(&self, resource: &str) -> Result<Option<LockInfo>>;

    /// 列出所有锁
    async fn list_locks(&self) -> Result<Vec<LockInfo>>;
}

/// 锁守卫，自动释放
pub struct LockGuard {
    resource: String,
    handle: LockHandle,
    lock_service: Arc<dyn DistributedLock>,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // 异步释放锁
        let lock_service = self.lock_service.clone();
        let handle = self.handle.clone();
        tokio::spawn(async move {
            let _ = lock_service.release(handle).await;
        });
    }
}

/// 第一阶段的本地锁实现
pub struct LocalDistributedLock {
    locks: Arc<RwLock<HashMap<String, LockInfo>>>,
}

impl DistributedLock for LocalDistributedLock {
    async fn acquire(&self, resource: &str, ttl: Duration) -> Result<LockGuard> {
        let mut locks = self.locks.write().await;

        if let Some(existing) = locks.get(resource) {
            if !existing.is_expired() {
                return Err(anyhow!("Resource {} already locked", resource));
            }
        }

        let handle = LockHandle::new();
        let lock_info = LockInfo::new(handle.clone(), ttl);
        locks.insert(resource.to_string(), lock_info);

        Ok(LockGuard {
            resource: resource.to_string(),
            handle,
            lock_service: Arc::new(self.clone()),
        })
    }

    // ... 其他方法实现
}
```

### 4. 事件总线接口

#### ClusterEventBus - 集群事件总线

```rust
/// 集群事件总线接口
#[async_trait]
pub trait ClusterEventBus: Send + Sync {
    /// 发布事件到整个集群
    async fn publish_cluster(&self, topic: &str, event: Event) -> Result<()>;

    /// 发布事件到本地节点
    async fn publish_local(&self, topic: &str, event: Event) -> Result<()>;

    /// 订阅集群事件
    async fn subscribe_cluster(&self, topic: &str) -> Result<Receiver<Event>>;

    /// 订阅本地事件
    async fn subscribe_local(&self, topic: &str) -> Result<Receiver<Event>>;

    /// 获取事件统计
    async fn statistics(&self) -> Result<EventStatistics>;

    /// 配置事件路由规则
    async fn configure_routing(&self, rules: Vec<RoutingRule>) -> Result<()>;
}

/// 第一阶段的本地事件总线
pub struct LocalEventBus {
    channels: Arc<RwLock<HashMap<String, Vec<Sender<Event>>>>>,
    stats: Arc<AtomicU64>,
}

impl ClusterEventBus for LocalEventBus {
    async fn publish_cluster(&self, topic: &str, event: Event) -> Result<()> {
        // 第一阶段等同于本地发布
        self.publish_local(topic, event).await
    }

    async fn publish_local(&self, topic: &str, event: Event) -> Result<()> {
        let channels = self.channels.read().await;
        if let Some(senders) = channels.get(topic) {
            for sender in senders {
                let _ = sender.send(event.clone()).await;
            }
        }
        self.stats.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    // ... 其他方法实现
}
```

## 数据结构定义

### 核心类型

```rust
/// 节点唯一标识
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn local() -> Self {
        Self(Uuid::from_bytes([0; 16])) // 特殊的本地节点ID
    }
}

/// 提案唯一标识
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub Uuid);

impl ProposalId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn immediate() -> Self {
        Self(Uuid::from_bytes([1; 16])) // 特殊的立即提案ID
    }
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub address: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub resources: ResourceCapacity,
    pub metadata: HashMap<String, String>,
}

/// 状态变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChange {
    WorkflowCreated {
        workflow_id: WorkflowId,
        definition: WorkflowDef,
    },
    WorkflowStarted {
        workflow_id: WorkflowId,
        execution_id: ExecutionId,
    },
    TaskStarted {
        task_id: TaskId,
        node_id: NodeId,
    },
    TaskCompleted {
        task_id: TaskId,
        result: TaskResult,
    },
    TaskFailed {
        task_id: TaskId,
        error: TaskError,
    },
    ResourceAllocated {
        resource_id: ResourceId,
        node_id: NodeId,
        allocation: ResourceAllocation,
    },
    ResourceReleased {
        resource_id: ResourceId,
        node_id: NodeId,
    },
    NodeJoined {
        node_id: NodeId,
        node_info: NodeInfo,
    },
    NodeLeft {
        node_id: NodeId,
        reason: String,
    },
}

/// 集群拓扑
#[derive(Debug, Clone)]
pub struct ClusterTopology {
    pub nodes: Vec<NodeInfo>,
    pub leader: Option<NodeId>,
    pub regions: HashMap<String, Vec<NodeId>>,
    pub partition_map: HashMap<PartitionId, NodeId>,
}

impl ClusterTopology {
    pub fn single_node(node: NodeInfo) -> Self {
        let node_id = node.id;
        Self {
            nodes: vec![node],
            leader: Some(node_id),
            regions: HashMap::new(),
            partition_map: HashMap::new(),
        }
    }
}
```

## 配置驱动的部署模式

### 系统配置

```rust
/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub deployment: DeploymentConfig,
    pub consensus: ConsensusConfig,
    pub networking: NetworkConfig,
    pub storage: StorageConfig,
}

/// 部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentConfig {
    SingleNode {
        node_id: Option<NodeId>,
    },
    Cluster {
        nodes: Vec<String>,
        bootstrap_timeout: Duration,
    },
    P2P {
        bootstrap_nodes: Vec<String>,
        discovery_interval: Duration,
    },
}

/// 共识配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub algorithm: ConsensusAlgorithm,
    pub timeout: Duration,
    pub batch_size: usize,
    pub leader_election_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    NoOp,              // 第一阶段：无共识
    Raft,              // 第二阶段：Raft共识
    Byzantine,         // 第三阶段：拜占庭容错
    PoS,               // 第四阶段：权益证明
}
```

## 服务容器和依赖注入

### 服务容器

```rust
/// 统一的服务容器
pub struct ServiceContainer {
    pub consensus: Arc<dyn ConsensusEngine>,
    pub node_manager: Arc<dyn NodeManager>,
    pub state_sync: Arc<dyn StateSynchronizer>,
    pub distributed_lock: Arc<dyn DistributedLock>,
    pub event_bus: Arc<dyn ClusterEventBus>,
    pub config: Arc<SystemConfig>,
}

impl ServiceContainer {
    /// 创建单机版本（第一阶段）
    pub fn new_single_node(config: SystemConfig) -> Self {
        let node_info = NodeInfo::local();

        Self {
            consensus: Arc::new(NoOpConsensus),
            node_manager: Arc::new(LocalNodeManager::new(node_info)),
            state_sync: Arc::new(LocalStateSynchronizer::new()),
            distributed_lock: Arc::new(LocalDistributedLock::new()),
            event_bus: Arc::new(LocalEventBus::new()),
            config: Arc::new(config),
        }
    }

    /// 创建集群版本（第二阶段）
    pub async fn new_cluster(config: SystemConfig) -> Result<Self> {
        match config.consensus.algorithm {
            ConsensusAlgorithm::Raft => {
                Ok(Self {
                    consensus: Arc::new(RaftConsensus::new(&config).await?),
                    node_manager: Arc::new(ClusterNodeManager::new(&config).await?),
                    state_sync: Arc::new(DistributedStateSynchronizer::new(&config).await?),
                    distributed_lock: Arc::new(EtcdDistributedLock::new(&config).await?),
                    event_bus: Arc::new(DistributedEventBus::new(&config).await?),
                    config: Arc::new(config),
                })
            }
            _ => Err(anyhow!("Unsupported consensus algorithm for cluster mode")),
        }
    }

    /// 创建P2P版本（第三阶段）
    pub async fn new_p2p(config: SystemConfig) -> Result<Self> {
        // TODO: 第三阶段实现
        todo!("Implement P2P mode in phase 3")
    }
}
```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_consensus() {
        let consensus = NoOpConsensus;
        let change = StateChange::WorkflowCreated {
            workflow_id: WorkflowId::new(),
            definition: WorkflowDef::default(),
        };

        let proposal_id = consensus.propose(change).await.unwrap();
        assert_eq!(proposal_id, ProposalId::immediate());

        let status = consensus.status(proposal_id).await.unwrap();
        assert_eq!(status, ProposalStatus::Committed);
    }

    #[tokio::test]
    async fn test_local_node_manager() {
        let node_info = NodeInfo::local();
        let manager = LocalNodeManager::new(node_info.clone());

        let nodes = manager.discover().await.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, node_info.id);
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_service_container_single_node() {
        let config = SystemConfig::single_node();
        let container = ServiceContainer::new_single_node(config);

        // 测试各个服务的集成
        let change = StateChange::WorkflowCreated {
            workflow_id: WorkflowId::new(),
            definition: WorkflowDef::default(),
        };

        // 通过共识层提案
        let proposal_id = container.consensus.propose(change.clone()).await.unwrap();

        // 通过状态同步广播
        container.state_sync.broadcast_change(change).await.unwrap();

        // 通过事件总线发布
        let event = Event::new("workflow.created", serde_json::to_value(change).unwrap());
        container.event_bus.publish_local("workflow", event).await.unwrap();
    }
}
```

## 演进路径

### 第一阶段 (0-12 个月): NoOp 实现

-   所有接口采用 NoOp/Local 实现
-   专注单机性能优化
-   建立完整的接口规范
-   为分布式测试做准备

### 第二阶段 (12-24 个月): 分布式实现

-   Raft 共识算法实现
-   真正的集群节点管理
-   etcd/Redis 分布式锁
-   网络状态同步

### 第三阶段 (24-36 个月): P2P 实现

-   libp2p 网络层
-   Byzantine 共识算法
-   去中心化节点发现
-   加密状态同步

### 第四阶段 (36 个月+): 区块链集成

-   权益证明共识
-   智能合约集成
-   跨链互操作
-   代币经济模型

这种设计确保了 FlowBuilder 可以从单机平滑演进到分布式和去中心化，每个阶段都有明确的接口和实现策略。
