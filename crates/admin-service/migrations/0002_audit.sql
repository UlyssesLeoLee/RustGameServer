-- admin-service migration 0002_audit_prev_hash_unique（per RGS-REV-007 AC5=CC1+CH3 / DEC-015 P1）
-- 55.13 增补：audit_log.prev_hash 加 UNIQUE 约束 —— 防止同一 prev_hash 被并发插入两次
-- 出现 hash 链分叉（结合 service 层 SELECT ... FOR UPDATE 锁 latest 行提供第二道防线）。

-- 注意：0001_init.sql 中 hash 字段已是 UNIQUE（hash = sha256(...) 唯一），
-- 此处 prev_hash 约束提供 read-then-append 并发防御的最终保险：若两个事务同时读到
-- 同一 latest 行（理论上 FOR UPDATE 已串行化，但作为深度防御仍建此约束），
-- UNIQUE(prev_hash) 将让第二笔 INSERT 失败而非产生链分叉。
ALTER TABLE audit_log
    ADD CONSTRAINT uq_audit_log_prev_hash UNIQUE (prev_hash);
