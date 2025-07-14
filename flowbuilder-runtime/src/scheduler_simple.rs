//! # 简化的调度器模块
//!
//! 提供基本的任务调度功能

use anyhow::Result;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    /// 低优先级
    Low = 1,
    /// 普通优先级
    #[default]
    Normal = 2,
    /// 高优先级
    High = 3,
    /// 紧急优先级
    Critical = 4,
}

/// 调度策略
#[derive(Debug, Clone)]
pub enum SchedulingStrategy {
    /// 先进先出 (FIFO)
    FirstInFirstOut,
    /// 优先级调度
    Priority,
    /// 轮询调度
    RoundRobin,
    /// 最短作业优先
    ShortestJobFirst,
}

/// 任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 简化的调度任务
#[derive(Clone)]
pub struct ScheduledTask {
    /// 任务ID
    pub id: Uuid,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: Option<String>,
    /// 优先级
    pub priority: Priority,
    /// 预估执行时间
    pub estimated_duration: Option<Duration>,
    /// 创建时间
    pub created_at: Instant,
    /// 开始时间
    pub started_at: Option<Instant>,
    /// 完成时间
    pub completed_at: Option<Instant>,
    /// 状态
    pub status: TaskStatus,
    /// 依赖的任务ID列表
    pub dependencies: Vec<Uuid>,
    /// 任务执行函数
    pub task_fn: Arc<dyn Fn() -> Result<()> + Send + Sync>,
    /// 任务元数据
    pub metadata: HashMap<String, String>,
}

impl ScheduledTask {
    /// 创建新的调度任务
    pub fn new(
        name: String,
        task_fn: Arc<dyn Fn() -> Result<()> + Send + Sync>,
        priority: Priority,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            priority,
            estimated_duration: None,
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            task_fn,
            metadata: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for ScheduledTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScheduledTask")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("status", &self.status)
            .finish()
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先级高的排在前面（数值大的优先级高）
        // BinaryHeap 是最大堆，所以我们让高优先级的任务有更大的排序值
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at)) // 创建时间早的优先
    }
}

/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 调度策略
    pub strategy: SchedulingStrategy,
    /// 任务超时时间
    pub task_timeout: Option<Duration>,
    /// 重试次数
    pub max_retries: u32,
    /// 重试延迟
    pub retry_delay: Duration,
    /// 是否启用依赖检查
    pub enable_dependency_check: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            strategy: SchedulingStrategy::Priority,
            task_timeout: Some(Duration::from_secs(300)),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            enable_dependency_check: true,
        }
    }
}

/// 调度器统计信息
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    /// 等待任务数
    pub pending_tasks: usize,
    /// 运行中任务数
    pub running_tasks: usize,
    /// 已完成任务数
    pub completed_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 总调度次数
    pub total_scheduled: usize,
    /// 平均执行时间
    pub avg_execution_time: Duration,
    /// 最大等待时间
    pub max_wait_time: Duration,
}

/// 任务调度器
pub struct TaskScheduler {
    /// 配置
    config: SchedulerConfig,
    /// 任务队列（优先级队列）
    task_queue: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    /// FIFO队列（用于FIFO策略）
    fifo_queue: Arc<Mutex<VecDeque<ScheduledTask>>>,
    /// 任务状态映射
    task_status: Arc<Mutex<HashMap<Uuid, ScheduledTask>>>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 统计信息
    stats: Arc<Mutex<SchedulerStats>>,
    /// 是否正在运行
    is_running: Arc<Mutex<bool>>,
}

impl TaskScheduler {
    /// 创建新的任务调度器
    pub fn new(config: SchedulerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        Self {
            config,
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            fifo_queue: Arc::new(Mutex::new(VecDeque::new())),
            task_status: Arc::new(Mutex::new(HashMap::new())),
            semaphore,
            stats: Arc::new(Mutex::new(SchedulerStats::default())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// 使用默认配置创建调度器
    pub fn with_default_config() -> Self {
        Self::new(SchedulerConfig::default())
    }

    /// 提交任务到调度器
    pub async fn submit_task(&self, task: ScheduledTask) -> Result<Uuid> {
        let task_id = task.id;
        let task_name = task.name.clone();

        // 更新统计信息
        {
            let mut stats = self.stats.lock().await;
            stats.pending_tasks += 1;
            stats.total_scheduled += 1;
        }

        // 将任务添加到状态映射
        {
            let mut task_status = self.task_status.lock().await;
            task_status.insert(task_id, task.clone());
        }

        // 根据调度策略添加到相应队列
        match self.config.strategy {
            SchedulingStrategy::FirstInFirstOut => {
                let mut fifo_queue = self.fifo_queue.lock().await;
                fifo_queue.push_back(task);
            }
            _ => {
                let mut task_queue = self.task_queue.lock().await;
                task_queue.push(task);
            }
        }

        println!("任务已提交: {task_name} (ID: {task_id})");
        Ok(task_id)
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: Uuid) -> Option<TaskStatus> {
        let task_status = self.task_status.lock().await;
        task_status.get(&task_id).map(|task| task.status.clone())
    }

    /// 获取调度器统计信息
    pub async fn get_stats(&self) -> SchedulerStats {
        let stats = self.stats.lock().await;
        stats.clone()
    }

    /// 获取下一个要执行的任务 (用于测试)
    pub async fn get_next_task(&self) -> Option<ScheduledTask> {
        match self.config.strategy {
            SchedulingStrategy::FirstInFirstOut => {
                let mut fifo_queue = self.fifo_queue.lock().await;
                fifo_queue.pop_front()
            }
            _ => {
                let mut task_queue = self.task_queue.lock().await;
                task_queue.pop()
            }
        }
    }

    /// 检查是否可以调度任务 (用于测试)
    pub async fn can_schedule_task(&self, task: &ScheduledTask) -> bool {
        // 检查依赖是否满足
        if self.config.enable_dependency_check && !self.check_dependencies(&task.dependencies).await
        {
            return false;
        }

        // 检查并发限制
        self.semaphore.available_permits() > 0
    }

    /// 检查任务依赖
    async fn check_dependencies(&self, dependencies: &[Uuid]) -> bool {
        let task_status = self.task_status.lock().await;

        for dep_id in dependencies {
            if let Some(dep_task) = task_status.get(dep_id) {
                if dep_task.status != TaskStatus::Completed {
                    return false;
                }
            } else {
                // 依赖任务不存在
                return false;
            }
        }

        true
    }

    /// 执行任务 (用于测试)
    pub async fn execute_task(&self, mut task: ScheduledTask) -> Result<()> {
        let task_id = task.id;
        let task_name = task.name.clone();

        // 获取执行许可
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("获取执行许可失败: {}", e))?;

        // 更新任务状态
        task.status = TaskStatus::Running;
        task.started_at = Some(Instant::now());

        {
            let mut task_status = self.task_status.lock().await;
            task_status.insert(task_id, task.clone());
        }

        // 更新统计信息
        {
            let mut stats = self.stats.lock().await;
            stats.pending_tasks = stats.pending_tasks.saturating_sub(1);
            stats.running_tasks += 1;
        }

        println!("任务开始执行: {task_name} (ID: {task_id})");

        // 执行任务
        let result = (task.task_fn)();

        // 更新任务状态
        let final_status = match result {
            Ok(()) => {
                println!("任务执行成功: {task_name}");
                TaskStatus::Completed
            }
            Err(e) => {
                println!("任务执行失败: {task_name} - {e}");
                TaskStatus::Failed
            }
        };

        task.status = final_status.clone();
        task.completed_at = Some(Instant::now());

        {
            let mut task_status = self.task_status.lock().await;
            task_status.insert(task_id, task);
        }

        // 更新统计信息
        {
            let mut stats = self.stats.lock().await;
            stats.running_tasks = stats.running_tasks.saturating_sub(1);

            match final_status {
                TaskStatus::Completed => stats.completed_tasks += 1,
                TaskStatus::Failed => stats.failed_tasks += 1,
                _ => {}
            }
        }

        Ok(())
    }

    /// 清理已完成的任务 (用于测试)
    pub async fn cleanup_completed_tasks(&self) {
        // 在简化版本中，任务在执行后立即更新状态，无需额外清理
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<()> {
        {
            let mut is_running = self.is_running.lock().await;
            if *is_running {
                return Err(anyhow::anyhow!("调度器已在运行"));
            }
            *is_running = true;
        }

        println!("任务调度器启动，策略: {:?}", self.config.strategy);

        // 简化的调度循环
        loop {
            {
                let is_running = self.is_running.lock().await;
                if !*is_running {
                    break;
                }
            }

            // 检查并调度新任务
            if let Some(task) = self.get_next_task().await {
                if self.can_schedule_task(&task).await {
                    let _ = self.execute_task(task).await;
                }
            }

            // 短暂休眠
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) -> Result<()> {
        {
            let mut is_running = self.is_running.lock().await;
            *is_running = false;
        }

        println!("任务调度器已停止");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_scheduler_basic_functionality() {
        let scheduler = TaskScheduler::with_default_config();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: "test_task".to_string(),
            description: Some("测试任务".to_string()),
            priority: Priority::Normal,
            estimated_duration: Some(Duration::from_millis(100)),
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            status: TaskStatus::Pending,
            dependencies: Vec::new(),
            task_fn: Arc::new(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
            metadata: HashMap::new(),
        };

        let task_id = scheduler.submit_task(task).await.unwrap();

        // 验证任务已提交
        assert_eq!(
            scheduler.get_task_status(task_id).await,
            Some(TaskStatus::Pending)
        );

        // 执行任务
        if let Some(task) = scheduler.get_next_task().await {
            scheduler.execute_task(task).await.unwrap();
        }

        // 验证任务完成
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(
            scheduler.get_task_status(task_id).await,
            Some(TaskStatus::Completed)
        );
    }

    #[tokio::test]
    async fn test_scheduler_priority_ordering() {
        let scheduler = TaskScheduler::with_default_config();

        // 创建不同优先级的任务
        let tasks = vec![
            (Priority::Low, "低优先级"),
            (Priority::High, "高优先级"),
            (Priority::Normal, "普通优先级"),
            (Priority::Critical, "紧急优先级"),
        ];

        for (priority, name) in tasks {
            let task = ScheduledTask {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: None,
                priority,
                estimated_duration: None,
                created_at: Instant::now(),
                started_at: None,
                completed_at: None,
                status: TaskStatus::Pending,
                dependencies: Vec::new(),
                task_fn: Arc::new(|| Ok(())),
                metadata: HashMap::new(),
            };

            scheduler.submit_task(task).await.unwrap();
        }

        // 获取任务，应该按优先级排序
        let first_task = scheduler.get_next_task().await.unwrap();
        assert_eq!(first_task.priority, Priority::Critical);

        let second_task = scheduler.get_next_task().await.unwrap();
        assert_eq!(second_task.priority, Priority::High);
    }
}
