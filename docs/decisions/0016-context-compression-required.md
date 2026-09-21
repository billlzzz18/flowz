# ADR-0016: Context Compression เป็น Required Component

**Status:** Accepted
**Date:** 2026-09-21

## Context

Workflow ที่มีหลาย items ที่สะสมผลลัพธ์จะทำให้ context window เต็ม Hermes แก้ปัญหานี้ด้วย dual compression system ที่ทำงานที่ 50% threshold และ micro-compaction หลังแต่ละ turn

## Decision

- ContextCompressor เป็น trait ที่ workflow engine ต้องมี
- ใช้ two-layer compression:
  - Preflight compression: ที่ 50% ของ context window (configurable)
  - Micro-compaction: หลังแต่ละ turn ที่เสร็จ
- ใช้ lightweight auxiliary model สำหรับ summarization
- Tail budget ratio: 20% ของ compressed context

```rust
#[async_trait::async_trait]
pub trait ContextCompressor: Send + Sync {
    async fn compress(
        &self,
        history: &mut Vec<Message>,
        target_tokens: usize,
    ) -> anyhow::Result<CompressionResult>;

    async fn micro_compact(
        &self,
        history: &mut Vec<Message>,
    ) -> anyhow::Result<()>;
}

pub struct CompressionResult {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub summary: Option<String>,
}
```

Integration points:
- Preflight compression: ตรวจสอบที่ 50% ของ context window ก่อน API call
- Micro-compaction: หลังแต่ละ turn ที่เสร็จ
- Auxiliary model: ใช้ lightweight model สำหรับ summarization

```rust
pub struct WorkflowEngine {
    pub compressor: Arc<dyn ContextCompressor>,
    pub context_threshold: f32,  // default 0.5
    pub tail_budget_ratio: f32,  // default 0.2
    // ...
}
```

## Consequences

### Positive

- Context window ไม่เต็ม
- Conversation ต่อได้ยาวขึ้น
- Cost ต่ำกว่าการส่ง context ทั้งหมด

### Negative

- ข้อมูลอาจสูญหายจากการ compress
- ต้องมี auxiliary model
- Complexity เพิ่มขึ้น

## Enforcement

- WorkflowEngine ต้องมี ContextCompressor
- Test: workflow ที่มี 100 items ต้องไม่ overflow
- Test: compression ทำงานที่ threshold ที่กำหนด