// codeblock.js - 轻量级代码高亮（不引入 highlighter 库）
// 设计文档: docs/03-详细设计书.md §2.7
// 实现注意: 先 Escape HTML, 再用 span 包裹（避免误伤 < > &）

import { escapeHtml } from './util.js';

const RULES = {
  csharp: [
    [/\b(var|void|int|float|bool|string|class|public|private|protected|static|new|return|if|else|for|while|do|switch|case|break|continue|true|false|null|this|using|namespace|get|set)\b/g, 'kw'],
    [/"([^"\\]|\\.)*"/g, 'str'],
    [/\/\/.*$/gm, 'cm'],
    [/\b\d+\.?\d*[fLuUlL]?\b/g, 'num'],
    [/@[A-Za-z_]\w*/g, 'pp']
  ],
  cpp: [
    [/\b(void|int|float|double|bool|char|long|short|unsigned|signed|const|static|class|struct|public|private|protected|virtual|new|delete|return|if|else|for|while|do|switch|case|break|continue|true|false|nullptr|auto|namespace|using|template|typename|enum|sizeof|operator)\b/g, 'kw'],
    [/"([^"\\]|\\.)*"/g, 'str'],
    [/\/\/.*$/gm, 'cm'],
    [/\b\d+\.?\d*[fLuUlL]?\b/g, 'num'],
    [/(#\s*(include|define|ifdef|ifndef|endif|pragma|if|else|elif)\b)/g, 'pp']
  ],
  gdscript: [
    [/\b(var|func|class|extends|if|elif|else|for|while|return|true|false|null|onready|export|signal|const|enum|static|pass|break|continue|self|in|not|and|or)\b/g, 'kw'],
    [/"([^"\\]|\\.)*"/g, 'str'],
    [/(^|\s)(#.+)$/gm, 'cm'],
    [/\b\d+\.?\d*\b/g, 'num']
  ],
  c: [
    [/\b(void|int|float|double|char|long|short|unsigned|signed|const|static|struct|enum|return|if|else|for|while|do|switch|case|break|continue|true|false|NULL|sizeof|typedef)\b/g, 'kw'],
    [/"([^"\\]|\\.)*"/g, 'str'],
    [/\/\/.*$/gm, 'cm'],
    [/\b\d+\.?\d*[fLuUlL]*\b/g, 'num'],
    [/(#\s*(include|define|ifdef|ifndef|endif|pragma|if|else|elif)\b)/g, 'pp']
  ]
};

export function renderCodeBlock(language, code) {
  const rules = RULES[language] || RULES.c;
  let html = escapeHtml(code);
  // Escape 后, 所有 < > & " ' 都被转义, 正则匹配安全
  for (const [regex, cls] of rules) {
    html = html.replace(regex, (m) => {
      // 防止重复嵌套 span
      if (m.startsWith('<span')) return m;
      return `<span class="tok-${cls}">${m}</span>`;
    });
  }
  return `<pre class="code code-${escapeHtml(language)}"><code>${html}</code></pre>`;
}
