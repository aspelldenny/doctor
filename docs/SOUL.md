# SOUL — doctor 3 hard lines

## Hard line 1 — MECHANICAL SOUND ONLY

Doctor chỉ làm việc MECHANICAL có oracle SOUND (compile, count, exact grep, path exist). KHÔNG ôm logic judgment.

Application:
- ✅ `lane-check` đếm dòng → SOUND
- ✅ `validate-map` check path exist → SOUND
- ✅ `rotate-check` đếm dòng → SOUND
- ✅ `runtime-scan` grep regex → SOUND
- ❌ "Phân loại entry doctrine vs operational" → PARTIAL, không phải việc doctor (rotate-archive Python pilot làm)
- ❌ "Judge phiếu có 'over-engineer' không" → judgment, không grep được

**Doctor là cái thước đo, không phải cái não. Đừng ép judgment thành hook giả → phán bừa.**

## Hard line 2 — RUST PORTABLE, CARGO INSTALL ONCE

Doctor PHẢI cài qua `cargo install --path .` 1 lần, sau đó mọi repo dùng được. KHÔNG per-project copy script.

Application:
- ✅ Single binary trong `~/.cargo/bin/doctor`
- ✅ Mọi project chỉ cần symlink agents / copy hooks → doctor binary đã sẵn sàng
- ❌ Per-project copy của doctor logic
- ❌ Python script duplicate logic ở mỗi repo

Pattern khớp 6 binary precedent (vps/ship/guard/quality-gate/advisory-cron/advisory-inbox).

## Hard line 3 — KHÔNG NUỐT PILOT THÍ NGHIỆM

Doctor làm phần SOUND. Khi gặp việc PARTIAL (judgment, format lạ, cần human decide), doctor STOP và defer sang tool khác (Python pilot).

Lý do (Claude Web round 8): nếu doctor ôm cả PARTIAL, em (Sếp) mất pilot vòng 2 thí nghiệm. Sếp mê Rust = cám dỗ thật, nhưng "giữ ranh: thứ mày MÊ thuộc về dụng cụ; thứ mày CẦN TEST thuộc về pilot; đừng để sở thích nuốt mục đích thí nghiệm".

Application cho rotate scenario:
- doctor `rotate-check` → đếm dòng, warn/block cap → SOUND, doctor làm.
- rotate-archive Python (pilot vòng 2) → phân loại entry, quyết cắt, format lạ → PARTIAL, doctor KHÔNG đụng.

**Câu hỏi vàng em (orchestrator) phải tự hỏi suốt:** "Làm Rust vì nó giải vấn đề test, hay vì em thích Rust?" Doctor = vì portable + gom-cargo-install. Test value sẽ ở Python pilot, KHÔNG ở doctor.

---

3 hard lines này là ranh giới doctor. Vượt 1 trong 3 → doctor đã trở thành cái khác, không phải doctor v2.2.
