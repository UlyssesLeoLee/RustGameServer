/**
 * expr.ts — Guard 表达式解析与求值 (无 eval / 无 new Function)
 *
 * spec `prototype_workbench.state_model` 要求 Guard 是**版本化的一等实体**,
 * 而不是散落在 payload 里的字符串。构建期离线门禁禁止 eval, 因此这里手写
 * 递归下降解析器 + AST 求值器。
 *
 * 支持语法:
 *   字面量      123 / 1.5 / 'text' / "text" / true / false / null
 *   路径        vars.retryCount / screen.panel / component.CMP-001.status
 *   逻辑        ! && ||
 *   比较        == != < <= > >=
 *   算术        + - * / %
 *   分组        ( )
 *
 * 任何解析失败 → 返回 { ok:false, error }, 调用方按"guard 不通过"处理,
 * 绝不静默当作 true (spec: 缺标比错标安全)。
 */

export type ExprValue = string | number | boolean | null | undefined;

export type Ast =
  | { t: 'lit'; v: ExprValue }
  | { t: 'var'; path: string[] }
  | { t: 'un'; op: string; a: Ast }
  | { t: 'bin'; op: string; a: Ast; b: Ast };

type TokKind = 'num' | 'str' | 'ident' | 'op' | 'punc';

interface Tok {
  kind: TokKind;
  value: string;
  pos: number;
}

const OPS2 = ['==', '!=', '<=', '>=', '&&', '||'];
const OPS1 = ['<', '>', '+', '-', '*', '/', '%', '!'];

export function tokenize(src: string): Tok[] {
  const out: Tok[] = [];
  let i = 0;
  while (i < src.length) {
    const ch = src[i];
    if (ch === ' ' || ch === '\t' || ch === '\n' || ch === '\r') { i++; continue; }
    if (ch === '"' || ch === "'") {
      const quote = ch;
      let j = i + 1;
      let buf = '';
      while (j < src.length && src[j] !== quote) {
        if (src[j] === '\\' && j + 1 < src.length) { buf += src[j + 1]; j += 2; }
        else { buf += src[j]; j++; }
      }
      if (j >= src.length) throw new Error(`未闭合的字符串 (pos ${i})`);
      out.push({ kind: 'str', value: buf, pos: i });
      i = j + 1;
      continue;
    }
    if (ch >= '0' && ch <= '9') {
      let j = i;
      while (j < src.length && ((src[j] >= '0' && src[j] <= '9') || src[j] === '.')) j++;
      out.push({ kind: 'num', value: src.slice(i, j), pos: i });
      i = j;
      continue;
    }
    if (/[A-Za-z_$]/.test(ch)) {
      let j = i;
      while (j < src.length && /[A-Za-z0-9_$.\-]/.test(src[j])) j++;
      out.push({ kind: 'ident', value: src.slice(i, j), pos: i });
      i = j;
      continue;
    }
    const two = src.slice(i, i + 2);
    if (OPS2.includes(two)) { out.push({ kind: 'op', value: two, pos: i }); i += 2; continue; }
    if (OPS1.includes(ch)) { out.push({ kind: 'op', value: ch, pos: i }); i++; continue; }
    if (ch === '(' || ch === ')') { out.push({ kind: 'punc', value: ch, pos: i }); i++; continue; }
    throw new Error(`无法识别的字符 '${ch}' (pos ${i})`);
  }
  return out;
}

class Parser {
  private i = 0;
  constructor(private toks: Tok[]) {}

  private peek(): Tok | undefined { return this.toks[this.i]; }
  private next(): Tok | undefined { return this.toks[this.i++]; }
  private expect(kind: TokKind, value?: string): Tok {
    const t = this.next();
    if (!t) throw new Error('表达式意外结束');
    if (t.kind !== kind) throw new Error(`期望 ${value ?? kind}, 实际 '${t.value}'`);
    if (value !== undefined && t.value !== value) throw new Error(`期望 '${value}', 实际 '${t.value}'`);
    return t;
  }

  parse(): Ast {
    const e = this.parseOr();
    if (this.i < this.toks.length) throw new Error(`表达式尾部有多余内容: '${this.toks[this.i].value}'`);
    return e;
  }

  private parseOr(): Ast {
    let left = this.parseAnd();
    while (this.peek()?.kind === 'op' && this.peek()!.value === '||') {
      this.next();
      left = { t: 'bin', op: '||', a: left, b: this.parseAnd() };
    }
    return left;
  }

  private parseAnd(): Ast {
    let left = this.parseNot();
    while (this.peek()?.kind === 'op' && this.peek()!.value === '&&') {
      this.next();
      left = { t: 'bin', op: '&&', a: left, b: this.parseNot() };
    }
    return left;
  }

  private parseNot(): Ast {
    if (this.peek()?.kind === 'op' && this.peek()!.value === '!') {
      this.next();
      return { t: 'un', op: '!', a: this.parseNot() };
    }
    return this.parseComparison();
  }

  private parseComparison(): Ast {
    const left = this.parseAdditive();
    const t = this.peek();
    if (t && t.kind === 'op' && ['==', '!=', '<', '<=', '>', '>='].includes(t.value)) {
      this.next();
      return { t: 'bin', op: t.value, a: left, b: this.parseAdditive() };
    }
    return left;
  }

  private parseAdditive(): Ast {
    let left = this.parseMultiplicative();
    for (;;) {
      const t = this.peek();
      if (t && t.kind === 'op' && (t.value === '+' || t.value === '-')) {
        this.next();
        left = { t: 'bin', op: t.value, a: left, b: this.parseMultiplicative() };
      } else return left;
    }
  }

  private parseMultiplicative(): Ast {
    let left = this.parseUnary();
    for (;;) {
      const t = this.peek();
      if (t && t.kind === 'op' && (t.value === '*' || t.value === '/' || t.value === '%')) {
        this.next();
        left = { t: 'bin', op: t.value, a: left, b: this.parseUnary() };
      } else return left;
    }
  }

  private parseUnary(): Ast {
    const t = this.peek();
    if (t && t.kind === 'op' && t.value === '-') {
      this.next();
      return { t: 'un', op: '-', a: this.parseUnary() };
    }
    return this.parsePrimary();
  }

  private parsePrimary(): Ast {
    const t = this.next();
    if (!t) throw new Error('表达式意外结束');
    if (t.kind === 'num') return { t: 'lit', v: Number(t.value) };
    if (t.kind === 'str') return { t: 'lit', v: t.value };
    if (t.kind === 'ident') {
      if (t.value === 'true') return { t: 'lit', v: true };
      if (t.value === 'false') return { t: 'lit', v: false };
      if (t.value === 'null') return { t: 'lit', v: null };
      return { t: 'var', path: t.value.split('.') };
    }
    if (t.kind === 'punc' && t.value === '(') {
      const e = this.parseOr();
      this.expect('punc', ')');
      return e;
    }
    throw new Error(`无法解析的记号 '${t.value}'`);
  }
}

/** 解析表达式; 失败返回 null (调用方按 guard 不通过处理) */
export function parseExpr(src: string): Ast | null {
  try {
    if (!src || !src.trim()) return null;
    return new Parser(tokenize(src)).parse();
  } catch {
    return null;
  }
}

/** 从 scope 取路径值 */
export function getPath(scope: Record<string, unknown>, path: string[]): ExprValue {
  if (!path.length) return undefined;
  let cur: unknown = scope;
  for (const seg of path) {
    if (cur === null || cur === undefined) return undefined;
    if (typeof cur !== 'object') return undefined;
    cur = (cur as Record<string, unknown>)[seg];
  }
  return cur as ExprValue;
}

function truthy(v: ExprValue): boolean {
  if (v === undefined || v === null || v === false) return false;
  if (v === 0) return false;
  if (v === '') return false;
  return true;
}

function num(v: ExprValue): number {
  if (typeof v === 'number') return v;
  if (typeof v === 'boolean') return v ? 1 : 0;
  if (v === undefined || v === null || v === '') return 0;
  const n = Number(v);
  return Number.isFinite(n) ? n : 0;
}

function looseEq(a: ExprValue, b: ExprValue): boolean {
  if (a === b) return true;
  if (a === null && b === undefined) return true;
  if (a === undefined && b === null) return true;
  if (typeof a === 'number' && typeof b === 'string') return a === num(b);
  if (typeof a === 'string' && typeof b === 'number') return num(a) === b;
  return false;
}

export function evalAst(ast: Ast, scope: Record<string, unknown>): ExprValue {
  switch (ast.t) {
    case 'lit': return ast.v;
    case 'var': return getPath(scope, ast.path);
    case 'un': {
      const v = evalAst(ast.a, scope);
      return ast.op === '!' ? !truthy(v) : -num(v);
    }
    case 'bin': {
      const a = evalAst(ast.a, scope);
      if (ast.op === '&&') return truthy(a) ? evalAst(ast.b, scope) : a;
      if (ast.op === '||') return truthy(a) ? a : evalAst(ast.b, scope);
      const b = evalAst(ast.b, scope);
      switch (ast.op) {
        case '==': return looseEq(a, b);
        case '!=': return !looseEq(a, b);
        case '<': return num(a) < num(b);
        case '<=': return num(a) <= num(b);
        case '>': return num(a) > num(b);
        case '>=': return num(a) >= num(b);
        case '+': return (typeof a === 'string' || typeof b === 'string')
          ? `${String(a ?? '')}${String(b ?? '')}`
          : num(a) + num(b);
        case '-': return num(a) - num(b);
        case '*': return num(a) * num(b);
        case '/': return num(b) === 0 ? NaN : num(a) / num(b);
        case '%': return num(b) === 0 ? NaN : num(a) % num(b);
        default: return undefined;
      }
    }
  }
}

export interface ExprResult {
  ok: boolean;
  value: ExprValue;
  error?: string;
}

/** 求值入口; 解析失败 → ok:false (绝不当作 true) */
export function evalExpr(src: string, scope: Record<string, unknown>): ExprResult {
  const ast = parseExpr(src);
  if (!ast) return { ok: false, value: false, error: `表达式无法解析: ${src}` };
  try {
    return { ok: true, value: evalAst(ast, scope) };
  } catch (e) {
    return { ok: false, value: false, error: `表达式求值失败: ${(e as Error).message}` };
  }
}
