// Quick verification: NoopMock / DbMock produce deprecation warnings
// Run with: cargo build -p rgs-testkit --example test_deprecated_warns
// Expected: 2 deprecation warnings emitted (per WF-1-55.31 retry 强约束)
//
// 注: 本 example 是 CI 锚定 artifact, 故意允许 deprecated 引用以验证
// deprecation 警告确实会触发. **业务 test crate 禁止**用本 pattern.

#![allow(deprecated)]

use rgs_testkit::mock::DbMock;
use rgs_testkit::mock::NoopMock;

fn main() {
    let m = NoopMock;
    let url = DbMock::mock_url(&m);
    println!("Mock URL: {}", url);
}
