//! shared-platform build.rs（per WBS v0.3 §2A.5 WF-1-54.3）
//! 编译 common.proto 到 OUT_DIR；6 域 crate 通过 tonic::include_proto!("common.v1") 引用
use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .build_server(false) // shared-platform 不生成 server trait（仅公共类型）
        .build_client(true)
        .compile_protos(
            &["proto/common/v1/common.proto"],
            &["proto", "../shared-platform/proto"],
        )?;
    Ok(())
}
