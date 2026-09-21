use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 테스트끼리 충돌하지 않도록 나노초 타임스탬프를 붙인 임시 디렉터리 경로를 만든다.
/// 실제 디렉터리 생성은 호출한 쪽에서 필요할 때만 수행한다.
pub(crate) fn unique_temp_dir(prefix: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    std::env::temp_dir().join(format!("wpygen-{prefix}-{suffix}"))
}
