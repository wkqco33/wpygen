use std::io::{IsTerminal, Write, stderr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// stderr 전용 터미널 로딩 스피너.
/// AI 생성과 같이 오래 걸리는 작업 동안 경과 시간과 진행 상황을 실시간으로 표시한다.
pub struct AiSpinner {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl AiSpinner {
    pub fn start(provider: &str, model: &str) -> Self {
        let is_term = stderr().is_terminal();
        let running = Arc::new(AtomicBool::new(true));

        if !is_term {
            // 파이프나 비-TTY 환경에서는 스피너 스레드 없이 메시지만 1회 출력
            eprintln!("AI 템플릿 생성 요청 중 (provider={provider}, model={model})...");
            return Self {
                running,
                handle: None,
            };
        }

        let running_clone = Arc::clone(&running);
        let provider = provider.to_string();
        let model = model.to_string();

        let handle = thread::spawn(move || {
            let start_time = Instant::now();
            let mut frame_idx = 0;

            while running_clone.load(Ordering::Relaxed) {
                let elapsed = start_time.elapsed().as_secs();
                let frame = FRAMES[frame_idx % FRAMES.len()];
                frame_idx += 1;

                let status_text = if elapsed < 10 {
                    format!("AI 템플릿 생성 요청 중 ({provider}/{model})... [{elapsed}s]")
                } else if elapsed < 30 {
                    format!("AI가 프로젝트 구조 및 코드를 작성 중입니다... [{elapsed}s]")
                } else if elapsed < 60 {
                    format!("AI가 전체 소스 코드와 단위 테스트를 생성하고 있습니다... [{elapsed}s]")
                } else {
                    format!("대규모 프로젝트 생성 중입니다 (잠시만 기다려주세요)... [{elapsed}s]")
                };

                // \x1b[2K: 현재 라인 전체 삭제, \r: 맨 앞으로 이동
                eprint!("\r\x1b[2K\x1b[36m{frame}\x1b[0m {status_text}");
                let _ = stderr().flush();

                thread::sleep(Duration::from_millis(80));
            }

            // 스피너 종료 시 줄을 깨끗이 비운다.
            eprint!("\r\x1b[2K");
            let _ = stderr().flush();
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    pub fn finish(mut self) {
        self.stop_internal();
    }

    fn stop_internal(&mut self) {
        if self.running.swap(false, Ordering::Relaxed)
            && let Some(handle) = self.handle.take()
        {
            let _ = handle.join();
        }
    }
}

impl Drop for AiSpinner {
    fn drop(&mut self) {
        self.stop_internal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_starts_and_stops_cleanly() {
        let spinner = AiSpinner::start("ollama", "test-model");
        thread::sleep(Duration::from_millis(150));
        spinner.finish();
    }
}
