# KittyPaw — Claude Code 규칙

## 테스트

- **TDD 필수**: 새 기능·버그 수정 시 실패 테스트 먼저 작성 → 구현 → 통과 순서
- 커밋 전: `cargo test --workspace` 통과 확인
- E2E 테스트: `tests/` 디렉토리 (Telegram 등 외부 의존 제외)

## 커밋

- 커밋 전 반드시 허락 구하기
- 커밋 메시지: 한국어 또는 영어, co-author 미포함

## 명령

```bash
cargo build --workspace          # 전체 빌드
cargo test --workspace           # 전체 테스트
cargo test -p kittypaw-engine    # 엔진 테스트만
cargo fmt --all                  # 포맷
cargo clippy --workspace         # 린트
```

## 아키텍처

- `kittypaw-core`: 공유 타입 (Config, Skill, Permission 등)
- `kittypaw-engine`: LLM 연동, 스킬 실행, 스케줄러
- `kittypaw-sandbox`: QuickJS 기반 JS 샌드박스
- `kittypaw-llm`: LLM provider 추상화
- `kittypaw-app`: macOS 데스크탑 앱 (Tauri)
- `kittypaw-tg`: Telegram 채널

## 참고

- [TASKS.md](TASKS.md) — 현재 작업 목록
- [VISION.md](VISION.md) — 제품 철학
- [DESIGN.md](DESIGN.md) — 아키텍처 설계
