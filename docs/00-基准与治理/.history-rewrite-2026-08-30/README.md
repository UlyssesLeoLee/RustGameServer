# 2026-08-30 Git 历史重写记录

## 事由

`docs/deploy/RGS-AI-HANDOFF-UPSTREAM-2026-08-30.md`(commit `96794c9`,重写后 `6fe32b8`)在提交时,commit message 中泄露了一个有效的 GitHub fine-grained PAT(`github_pat_11AHE3X4...`,93 字符,`Actions: Read/Write` + `Contents: Read` 权限),该 repo 为公开仓库(`UlyssesLeoLee/RustGameServer`)。同一 token 出现在另外 2 个更早的 commit message 中(GHCR pipeline 相关落档)。

## 处置流程

1. 用户已在 GitHub 侧撤销该 token。
2. `git bundle create --all` 全量备份(含泄露 token 的原始历史)。
3. `git-filter-repo --replace-message` 重写全部 3 处泄露的 commit message,用固定占位串替换 token。
4. `git-filter-repo` 默认会 normalize 所有 commit message 的空白/换行,导致重写波及全部 705 个可达 commit(不仅是 3 个直接修改的),`.git/filter-repo/commit-map` 因此记录了 711 条 old→new 映射(含少量历史分支上的 commit)。
5. 用户 `git push --force origin main` 完成,`origin/main` 新 HEAD = `6fe32b8`。
6. 用 `commit-map.txt`(本目录,从 `.git/filter-repo/commit-map` 复制而来,重写内容字节完全一致,仅 commit hash 变化)对全仓库 138 个文本文件(.md/.yaml/.yml/.ps1/.sh/.txt,含 `.worktrees/` 下未跟踪的 handoff 笔记)做了长度保持的 commit hash 引用批量替换,消除因历史重写产生的悬空引用。

## 关键约束(向后适用)

- **禁止**在任何 commit message / 文档 / 日志中打印 GHCR PAT 明文,一律通过 `$env:GHCR_PAT` 环境变量引用。
- `commit-map.txt` 是仅有的完整 old→new 711 条映射来源(`.git/filter-repo/` 内容不受 `git gc` 保证保留),后续如发现遗漏的旧 hash 引用,以此文件为准解析。
