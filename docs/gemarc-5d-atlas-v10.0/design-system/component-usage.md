# Component Usage (ja-JP)

> 参照位置づけ: 審査参考のみ。公式テンプレート・公式認証ではない。

## palette_policy
低彩度の背景 + 識別性の高いが抑制された状態色。**業務ドメイン色をエラー色として再利用しない**。
明色 / 高コントラストeduGreš モードに対応。赤緑のみで状態を表現しない。

## layout_contract
- ページ: 上部=プロジェクト/Revision/検索/次元ナビ、左=ディレクトリ、中央=キャンバス、右=インスペクタ、下=4D/5D タイムライン
- Prototype は独立タブ
- 間隔: 4px 基準の 4/8/12/16/24/32
- サイズ: header 60 / left 256 / right 360 / min-canvas 480 / timeline 136 / control 36

## typography
見出し 20/28、小見出し 16/24、本文 14/22、ラベル 13/20、キャプション 12/18。
数値は tabular-nums。CJK 字形フォールバック対応。

## controls
凡例・ツールボタン・ラベルは理解可能な title / tooltip を持つ。
アイコンが既に表現している情報と同じ文言を繰り返さない。

## accessibility
WCAG 2.2 AA。キーボード/フォーカス/コントラスト/意味的コントロール/Reduced Motion/200% 拡大。
図の機能はマウス専用にしない。
