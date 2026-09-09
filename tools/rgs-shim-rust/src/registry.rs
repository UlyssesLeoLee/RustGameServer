// Cmd registry (per 9/9 13:50 JST v0.2 模式, Rust 重写 v0.3.0)
// 6 个内置 cmd; 后续 worker 扩自己域时, 加新表项 + handler 函数
//
// 设计: handler 取 owned Vec<u8> + Arc<RgsClient> (clone) + cmd,
// 返回 Pin<Box<dyn Future + Send>> 不绑 lifetime, 避免 HRTB 复杂度
// & self 的 lifetime 也不进 future (entry.handler 是 fn pointer, 不是闭包)

use crate::handlers;
use crate::rgs::RgsClient;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

// 注意: 返回 future 不能 bind lifetime, 所以 future bound = 'static
// (handler 内部 clone Arc, 不持有外部引用)
pub type AsyncHandler = fn(
    u16,                 // cmd
    Vec<u8>,             // payload owned
    Arc<RgsClient>,      // rgs client shared
) -> Pin<Box<dyn Future<Output = handlers::Response> + Send>>;

pub struct CmdEntry {
    pub handler: AsyncHandler,
    #[allow(dead_code)]
    pub name: &'static str,
}

pub struct Registry {
    map: std::collections::HashMap<u16, CmdEntry>,
}

impl Registry {
    pub fn new() -> Self {
        let mut map = std::collections::HashMap::new();
        map.insert(10101, CmdEntry { handler: handlers::handle_register, name: "register" });
        map.insert(10102, CmdEntry { handler: handlers::handle_enter_server, name: "enter_server" });
        map.insert(10103, CmdEntry { handler: handlers::handle_enter_server, name: "enter_server (alias)" });
        map.insert(10200, CmdEntry { handler: handlers::handle_map_enter, name: "map_enter" });
        map.insert(10400, CmdEntry { handler: handlers::handle_heartbeat, name: "heartbeat (RGS 5 域 HealthCheck)" });
        map.insert(11001, CmdEntry { handler: handlers::handle_role_list, name: "role_list (RGS player ListPlayers)" });
        Registry { map }
    }

    // dispatch 不 borrow self, 只读 map 是 Copy (HashMap 的 get 返回 Option<&V>)
    // 改写: clone Arc + 转移 ownership 进 future, 避免 self 生命周期进入 future
    pub fn dispatch(
        &self,
        cmd: u16,
        payload: Vec<u8>,
        rgs: Arc<RgsClient>,
    ) -> Pin<Box<dyn Future<Output = handlers::Response> + Send>> {
        match self.map.get(&cmd) {
            Some(entry) => (entry.handler)(cmd, payload, rgs),
            None => {
                tracing::warn!(cmd, "stub (no handler)");
                Box::pin(async move { handlers::Response { cmd, payload: vec![] } })
            }
        }
    }

    pub fn list(&self) -> Vec<u16> {
        let mut v: Vec<u16> = self.map.keys().copied().collect();
        v.sort();
        v
    }
}
