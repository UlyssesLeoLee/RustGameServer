;; draw_card_probability v1.1.0 (per RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §2.2 PoC)
;;
;; 抽卡概率计算 WASM plugin:
;; - 基础 SSR 概率 5%
;; - pity 加成: 每次未出 SSR +0.5% (上限 95%)
;; - 80 次保底: 100% 出
;;
;; 输入: linear memory i32 pointer (指向 pity_count i32 LE)
;; 输出: i32 概率 × 10000 (e.g. 0.05 = 500, 1.00 = 10000)
;;
;; 编译: wat2wasm draw_card_probability_v1.1.0.wat -o draw_card_probability_v1.1.0.wasm
;; 部署: 见 README.md §4

(module
  (memory (export "memory") 1)

  ;; compute 导出 (per function-plane/src/wasm_host.rs L9.5)
  ;; input: $ptr = linear memory offset (pity_count i32 LE)
  ;;        $len = 4 (pity_count i32 长度)
  ;; output: probability × 10000
  (func (export "compute") (param $ptr i32) (param $len i32) (result i32)
    ;; 1. 读 pity_count
    local.get $ptr
    i32.load
    local.set $pity

    ;; 2. 判定: pity >= 80 ?
    local.get $pity
    i32.const 80
    i32.ge_s
    (if
      (then
        ;; 保底: 100% = 10000
        i32.const 10000
        return
      ))

    ;; 3. 计算: 500 + pity * 50 (= 5% + pity * 0.5%)
    i32.const 500
    local.get $pity
    i32.const 50
    i32.mul
    i32.add
  )
)
