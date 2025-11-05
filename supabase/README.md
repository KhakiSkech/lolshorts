# LoLShorts Supabase Database Setup

Supabase 데이터베이스 마이그레이션 파일 및 설정 가이드입니다.

---

## 🎯 아키텍처 설계 원칙

### Local-First Architecture (로컬 우선 아키텍처)

**데이터베이스 (Supabase)**: 인증 & 결제만
- ✅ 사용자 인증 (Supabase Auth)
- ✅ 라이선스 관리 (FREE/PRO 티어)
- ✅ 결제 내역 (Toss Payments)

**로컬 스토리지 (사용자 PC)**: 게임 데이터 전체
- ✅ 게임 녹화 영상 (mp4)
- ✅ 게임 이벤트 (JSON)
- ✅ 추출된 클립 (mp4)
- ✅ 합성 영상 (mp4)
- ✅ 스크린샷 (jpg/png)

**장점**:
- ⚡ **빠른 속도**: 로컬 파일 읽기/쓰기
- 💾 **무제한 저장**: DB 용량 제한 없음
- 🔒 **프라이버시**: 사용자 데이터가 로컬에만 존재
- 💰 **비용 절감**: DB/Storage 비용 최소화

---

## 📋 마이그레이션 순서

마이그레이션은 **반드시 순서대로** 실행해야 합니다:

### 1. Licenses (인증 & 라이선스)
```bash
001_create_licenses_table.sql
```

**설명**: 사용자 라이선스 관리 (FREE/PRO 티어)

**주요 기능**:
- 자동 FREE 티어 생성 (신규 회원가입 시)
- Toss Payments 결제 추적
- 라이선스 만료 자동 처리

**테이블 구조**:
```sql
licenses (
    id UUID,
    user_id UUID -> auth.users(id),
    tier TEXT ('FREE' | 'PRO'),
    status TEXT ('ACTIVE' | 'EXPIRED' | 'CANCELLED'),

    -- Toss Payments
    toss_customer_id TEXT,
    toss_billing_key TEXT,
    toss_subscription_id TEXT,

    created_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ
)
```

---

### 2. Toss Payments (결제 내역)
```bash
002_create_toss_payments.sql
```

**설명**: Toss Payments 결제 및 구독 관리

**주요 기능**:
- 결제 요청/승인/취소 추적
- 월/연 구독 자동 결제
- Webhook 데이터 저장
- 결제 성공 시 자동 PRO 업그레이드

**테이블 구조**:
```sql
toss_payments (
    id UUID,
    user_id UUID -> auth.users(id),
    license_id UUID -> licenses(id),

    -- Toss 결제 정보
    payment_key TEXT,          -- Toss 결제 키
    order_id TEXT,             -- 주문 ID
    transaction_id TEXT,       -- 거래 ID

    amount INTEGER,            -- 결제 금액 (원)
    method TEXT,               -- 결제 수단
    status TEXT,               -- 결제 상태

    -- 구독 정보
    is_subscription BOOLEAN,
    subscription_period TEXT,  -- MONTHLY | YEARLY
    next_billing_date TIMESTAMPTZ,

    requested_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ
)
```

**자동 트리거**:
- ✅ 결제 성공 → 라이선스 PRO로 업그레이드
- ✅ 결제 취소 → 라이선스 FREE로 다운그레이드

---

## 🚀 실행 방법

### 1. Supabase CLI 사용 (권장)
```bash
# Supabase 프로젝트 초기화
supabase init

# 로컬 Supabase 시작
supabase start

# 마이그레이션 적용
supabase db push

# 또는 개별 마이그레이션 실행
supabase db execute --file supabase/migrations/001_create_licenses_table.sql
supabase db execute --file supabase/migrations/002_create_toss_payments.sql
```

### 2. Supabase Dashboard 사용
1. https://app.supabase.com 접속
2. 프로젝트 선택
3. **SQL Editor** 메뉴 이동
4. 각 마이그레이션 파일 내용 복사/붙여넣기
5. **순서대로** 실행 (001 → 002)

### 3. 로컬 PostgreSQL 사용
```bash
# PostgreSQL 접속
psql -U postgres -d lolshorts

# 마이그레이션 실행
\i supabase/migrations/001_create_licenses_table.sql
\i supabase/migrations/002_create_toss_payments.sql
```

---

## 🔐 Row Level Security (RLS)

모든 테이블에 RLS가 활성화되어 있습니다:

### 기본 정책
- **SELECT**: 사용자 본인의 데이터만 조회
- **INSERT**: 사용자 본인의 데이터만 생성
- **UPDATE**: 백엔드(webhook)만 가능
- **DELETE**: 사용자 본인의 데이터만 삭제

---

## 📊 데이터베이스 다이어그램

```
auth.users (Supabase Auth)
    ↓
licenses (1:1)
    ↓
toss_payments (1:N)
```

**로컬 스토리지 구조** (DB에 저장 안 함):
```
C:\Users\{username}\AppData\Local\LoLShorts\
└── games\
    └── {game_id}\
        ├── metadata.json         # 게임 정보, 이벤트, KDA
        ├── recording.mp4         # 전체 게임 녹화
        ├── clips\
        │   ├── pentakill_420s.mp4
        │   └── baron_steal_1200s.mp4
        ├── screenshots\
        │   └── thumbnail.jpg
        └── compositions\
            └── highlight_montage.mp4
```

---

## 🧪 테스트 데이터 삽입

마이그레이션 완료 후 테스트:

```sql
-- 1. 테스트 라이선스 확인 (자동 생성됨)
SELECT * FROM licenses WHERE user_id = auth.uid();

-- 2. 테스트 결제 생성
INSERT INTO toss_payments (
    user_id,
    license_id,
    payment_key,
    order_id,
    amount,
    method,
    status,
    is_subscription,
    subscription_period
)
VALUES (
    auth.uid(),
    (SELECT id FROM licenses WHERE user_id = auth.uid()),
    'test_payment_key_123',
    'order_20250105_001',
    9900,
    '카드',
    'DONE',
    TRUE,
    'MONTHLY'
)
RETURNING *;

-- 3. 라이선스 PRO로 업그레이드 확인
SELECT * FROM licenses WHERE user_id = auth.uid();

-- 4. 결제 내역 조회
SELECT * FROM get_user_payment_history(auth.uid(), 10);
```

---

## 🛠️ 유용한 SQL 함수

### 라이선스 확인
```sql
-- 라이선스 유효성 확인
SELECT is_license_valid('{license_id}');
```

### 결제 내역 조회
```sql
-- 사용자 결제 내역 (최근 10개)
SELECT * FROM get_user_payment_history(auth.uid(), 10);
```

---

## 📝 마이그레이션 롤백

문제 발생 시 역순으로 롤백:

```sql
-- 2. Toss Payments 테이블 삭제
DROP TABLE IF EXISTS toss_payments CASCADE;
DROP FUNCTION IF EXISTS process_toss_payment_success() CASCADE;
DROP FUNCTION IF EXISTS process_toss_payment_cancel() CASCADE;
DROP FUNCTION IF EXISTS get_user_payment_history(UUID, INT) CASCADE;

-- 1. Licenses 테이블 삭제
DROP TABLE IF EXISTS licenses CASCADE;
DROP FUNCTION IF EXISTS create_default_license() CASCADE;
DROP FUNCTION IF EXISTS is_license_valid(UUID) CASCADE;
```

---

## 🔄 환경 변수 설정

`.env` 파일에 Supabase 및 Toss Payments 정보 추가:

```bash
# Supabase 설정
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Toss Payments 설정
TOSS_CLIENT_KEY=test_ck_...  # 테스트: test_ck_... | 실제: live_ck_...
TOSS_SECRET_KEY=test_sk_...  # 테스트: test_sk_... | 실제: live_sk_...

# 로깅
RUST_LOG=info
```

---

## 💳 Toss Payments 연동 가이드

### 1. 테스트 계정 생성
```bash
# Toss Payments 개발자 센터
https://developers.tosspayments.com/

# 테스트 키 발급 (즉시)
- Client Key: test_ck_...
- Secret Key: test_sk_...
```

### 2. 월 구독 결제 플로우
```typescript
// Frontend: 결제 요청
const response = await invoke('create_toss_payment', {
  amount: 9900,
  orderName: 'LoLShorts PRO 월 구독',
  period: 'MONTHLY',
});

// Toss Payments 결제창 리다이렉트
window.location.href = response.checkout_url;

// Backend: Webhook 수신 (결제 성공)
POST /api/toss/webhook
→ toss_payments.status = 'DONE'
→ licenses.tier = 'PRO'
→ licenses.expires_at = NOW() + 1 month
```

### 3. 결제 취소 플로우
```typescript
// User 또는 Admin이 구독 취소
await invoke('cancel_toss_subscription', {
  payment_key: 'xxx',
});

// Webhook 수신 (취소 완료)
→ toss_payments.status = 'CANCELED'
→ licenses.tier = 'FREE'
→ licenses.expires_at = NULL
```

---

## 📚 참고 자료

- [Supabase Documentation](https://supabase.com/docs)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Row Level Security Guide](https://supabase.com/docs/guides/auth/row-level-security)
- [Toss Payments API Docs](https://docs.tosspayments.com/reference)
- [Toss Payments 정기결제](https://docs.tosspayments.com/guides/payment-widget/integration#정기결제)

---

## ✅ 마이그레이션 체크리스트

개발 완료 후 아래 순서대로 진행:

- [ ] 1. Supabase 프로젝트 생성
- [ ] 2. `.env` 파일 설정
- [ ] 3. `001_create_licenses_table.sql` 실행
- [ ] 4. `002_create_toss_payments.sql` 실행
- [ ] 5. Toss Payments 테스트 계정 생성
- [ ] 6. 테스트 결제 데이터 삽입 및 확인
- [ ] 7. 애플리케이션 연동 테스트
- [ ] 8. 사업자 등록 후 Toss Payments 실 계정 전환
- [ ] 9. 실제 결제 테스트

---

**작성일**: 2025-11-05
**프로젝트**: LoLShorts v0.1
**아키텍처**: Local-First (DB: Auth+Payment only, 게임 데이터: Local JSON)
**결제**: Toss Payments (월/연 구독)
