# KittyPaw Tasks

## Completed

### Skill Platform — Phase 0~4 ✅
- [x] Phase 0: SkillResolver (샌드박스 실제 데이터 반환)
- [x] Phase 1: 패키지 포맷 + 매니저 + executor 브릿지
- [x] Phase 2: File.read/write, Telegram.sendDocument, Env.get
- [x] Phase 3: GUI 스킬 갤러리 + 설정 위자드 (Dioxus)
- [x] Phase 4: 예제 패키지 5개 + 자동 번들 설치

### GUI: Tauri → Dioxus 전환 ✅
- [x] Tauri + SvelteKit 삭제 (~24k LOC)
- [x] Dioxus 0.6 순수 Rust GUI (~470 LOC)
- [x] `cargo run -p kittypaw-gui` 한 줄 실행

### Foundation 기반 기능 4개 ✅
- [x] OS keychain 시크릿 관리 (`keyring` crate)
- [x] 멀티 프로바이더 LLM (OpenAI + Claude + LlmRegistry)
- [x] Web.search / Web.fetch 샌드박스 프리미티브
- [x] 스킬 체이닝 (`[[chain]]` + prev_output 전달)

## In Progress

### Phase 5: 폴리시 + 배포
- [ ] 에러 처리, 온보딩, 문서
- [ ] GitHub 기반 배포 (registry index + `kittypaw install github:user/repo`)

## Backlog

### 🔴 P0: ASCII 데모 (VHS)
- **목표**: kittypaw의 핵심 가치를 30–60초 터미널 애니메이션으로 전달
- **도구**: [VHS](https://github.com/charmbracelet/vhs) (`.tape` → GIF/SVG)
- **시나리오**:
  - A: "30초 스킬 생성" — `kittypaw teach` → 코드 생성 → dry-run → 승인
  - B: "GUI 갤러리 원클릭 설치" — 스킬 갤러리 브라우징
  - C: "웹 검색 + 팩트체크" — Phase 6+ 비전 데모
- **산출물**: GIF (README용) + SVG (웹사이트용)
- **연구 문서**: `docs/research-2026-03-28.md` §5

### 🟠 P1: 모델 라우팅 (Phase 7)
- **참고**: [model-router.ts](https://github.com/umgbhalla/pi-config/blob/main/extensions/model-router.ts), [pi-prompt-template-model](https://github.com/nicobailon/pi-prompt-template-model)
- **연구 문서**: `docs/research-2026-03-28.md` §2.2, §2.4
- [ ] `kittypaw.toml`에 `[models]` 섹션 — 복수 프로바이더/모델 등록
- [ ] `package.toml`에 `model` 필드 — 스킬별 모델 지정
- [ ] teach loop 키워드 분류기 — 스킬 유형별 자동 모델 선택
  - `automation` → Haiku급, `analysis` → Sonnet급, `integration` → Opus급
- [ ] 2단계 신뢰도 게이팅 (high=자동, medium=추천만)
- [ ] `kittypaw-llm`에 OpenAI API 클라이언트 추가

### 🟠 P1: 웹 검색 + 콘텐츠 프리미티브 (Phase 6)
- **참고**: [pi-web-access](https://github.com/nicobailon/pi-web-access)
- **연구 문서**: `docs/research-2026-03-28.md` §2.1
- [ ] `Web.search(query, options)` 샌드박스 프리미티브
  - Exa (API 키 불필요) → 커스텀 폴백
- [ ] `Web.fetch(url)` — URL → 마크다운 추출
- [ ] GUI 검색 큐레이션 — 결과 선택 → 스킬 주입
- [ ] 프로바이더 폴백 체인 설정 (`kittypaw.toml`)

### 🟡 P2: 스킬 체이닝 (Phase 9)
- **참고**: [pi-prompt-template-model](https://github.com/nicobailon/pi-prompt-template-model) chain/parallel 패턴
- **연구 문서**: `docs/research-2026-03-28.md` §2.2
- [ ] `package.toml`에 `chain` 필드 (순차: `->`, 병렬: `parallel()`)
- [ ] 모델 로테이션 — 체인 단계마다 다른 모델
- [ ] `converge` 모드 — 변경 없으면 조기 종료
- [ ] 컨텍스트 전달 — 이전 단계 출력 → 다음 단계 입력

### 🟡 P2: AI 비서 프리셋
- **참고**: @ginipigi 아티클 (2026-03-28)
- **연구 문서**: `docs/research-2026-03-28.md` §2.6
- [ ] 지침 템플릿 시스템 (캐릭터 + 말투 + 배경지식 + 팩트체크 규칙)
- [ ] 팩트체크 파이프라인 — 복수 LLM 교차검증 스킬
- [ ] 콘텐츠 회고 스킬 — 데이터 → 패턴 분석 → 전략 제안

### 🟢 P3: 자율 최적화 루프 (Phase 8)
- **참고**: [pi-autoresearch](https://github.com/davebcn87/pi-autoresearch)
- **연구 문서**: `docs/research-2026-03-28.md` §2.3
- [ ] `kittypaw optimize <skill> --metric <name>` CLI
- [ ] 최적화 루프: 코드 수정 → 벤치마크 → 유지/리버트
- [ ] `optimization.jsonl` + `optimization.md` (세션 재개)
- [ ] 신뢰도 점수 (MAD 기반)
- [ ] `checks.sh` — 최적화 전 정합성 검사

### 🟢 P3: 한국 특화 스킬 패키지
- **참고**: [k-skill](https://github.com/NomaDamas/k-skill)
- **연구 문서**: `docs/research-2026-03-28.md` §2.5
- [ ] SRT/KTX 예약, 배송 조회, 미세먼지 등
- [ ] sops + age 시크릿 관리 패턴 도입
- [ ] `~/.kittypaw/skills/` 글로벌 설치 경로

### 🟢 P3: /daily 모닝 브리핑
- **연구 문서**: `docs/research-2026-03-28.md` §4, 메모리 `project_daily_workflow.md`
- [ ] Todoist CLI + Obsidian Tasks 통합
- [ ] Google Calendar 미팅 조회 → 일지 추가
- [ ] 미팅 노트 자동 생성 + 백링크
- [ ] (아이디어) Flex 출근 이벤트 트리거
