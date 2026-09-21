use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 테스트끼리 충돌하지 않는 임시 디렉터리 경로를 만든다. 타임스탬프는 병렬 실행에서
/// 같은 값이 나올 수 있으므로 프로세스 ID와 호출 순번을 함께 붙인다. 실제 디렉터리
/// 생성은 호출한 쪽에서 필요할 때만 수행한다.
pub(crate) fn unique_temp_dir(prefix: &str) -> PathBuf {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);

    std::env::temp_dir().join(format!(
        "wpygen-{prefix}-{}-{nanos}-{sequence}",
        std::process::id()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn generated_paths_are_unique_even_when_called_rapidly() {
        let paths: HashSet<PathBuf> = (0..64).map(|_| unique_temp_dir("unique")).collect();

        assert_eq!(paths.len(), 64);
    }
}
