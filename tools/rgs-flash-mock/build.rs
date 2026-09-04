// rgs-flash-mock build.rs
//
// v0.1 stub 模式: handlers 返 JSON 文档化 RGS routing, 不实际用 gRPC client
// v0.2+ 加回 tonic_build (需要修复 common.proto 路径解析, per `shared-platform/proto` include path)
//
// per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §1.2 5-10 sprint 路线图:
//   - W1 (v0.1, 本 turn): scaffold + 22 RPC stub 模式
//   - W2 (v0.2): 加 7 域 gRPC client (player/economy/match/social/admin/card/gm-backend)
//                   + 修复 proto 路径 + 加回 tonic-build

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // v0.1: no-op (待 v0.2 加回 tonic-build, 修复 common.proto 路径解析)
    // v0.2 实现 (待办):
    //   let proto_paths = &[ ... 7 域 proto ... ];
    //   let include_paths = &["../../crates", "../../crates/shared-platform/proto"];
    //   tonic_build::configure()
    //       .build_server(false)
    //       .build_client(true)
    //       .compile_protos(proto_paths, include_paths)?;
    Ok(())
}
