import { createHash } from 'node:crypto';
import type { Compiler } from '../Compiler';
import { Compilation } from '../Compilation';

export interface ExtractInlineWorkerPluginOptions {
  /**
   * Filename template used for extracted worker sources.
   *
   * @default 'workers/[contenthash:8].js'
   */
  filename?: string;
  /**
   * Only extract inline worker sources at least this many bytes long.
   *
   * @default 10000
   */
  minSize?: number;
  /**
   * Public path used by the inline `importScripts` bootstrap. By default the
   * plugin uses `output.publicPath` when it is a fixed string.
   */
  publicPath?: string;
}

interface StringLiteral {
  end: number;
  value: string;
}

interface InlineWorkerSource extends StringLiteral {
  start: number;
}

const SIMPLE_ESCAPES: Record<string, string> = {
  b: '\b',
  f: '\f',
  n: '\n',
  r: '\r',
  t: '\t',
  v: '\v',
  '0': '\0',
};

function readStringLiteral(
  code: string,
  start: number,
): StringLiteral | undefined {
  const quote = code[start];
  if (quote !== '"' && quote !== "'") return;

  let value = '';
  for (let index = start + 1; index < code.length; index++) {
    const char = code[index];
    if (char === quote) return { end: index + 1, value };
    if (char !== '\\') {
      value += char;
      continue;
    }

    const escaped = code[++index];
    if (escaped === undefined) return;
    if (escaped === '\n') continue;
    if (escaped === '\r') {
      if (code[index + 1] === '\n') index++;
      continue;
    }
    if (escaped === 'x') {
      const hex = code.slice(index + 1, index + 3);
      if (!/^[0-9a-f]{2}$/i.test(hex)) return;
      value += String.fromCharCode(Number.parseInt(hex, 16));
      index += 2;
      continue;
    }
    if (escaped === 'u') {
      if (code[index + 1] === '{') {
        const close = code.indexOf('}', index + 2);
        if (close < 0) return;
        const hex = code.slice(index + 2, close);
        if (!/^[0-9a-f]+$/i.test(hex)) return;
        value += String.fromCodePoint(Number.parseInt(hex, 16));
        index = close;
      } else {
        const hex = code.slice(index + 1, index + 5);
        if (!/^[0-9a-f]{4}$/i.test(hex)) return;
        value += String.fromCharCode(Number.parseInt(hex, 16));
        index += 4;
      }
      continue;
    }
    value += SIMPLE_ESCAPES[escaped] ?? escaped;
  }
}

function findInlineWorkerSources(
  code: string,
  minSize: number,
): InlineWorkerSource[] {
  const sources: InlineWorkerSource[] = [];
  for (let index = 0; index < code.length; index++) {
    const quote = code[index];
    if (quote !== '"' && quote !== "'") continue;

    const literal = readStringLiteral(code, index);
    if (!literal) continue;
    const start = index;
    index = literal.end - 1;
    if (Buffer.byteLength(literal.value) < minSize) continue;

    const before = code.slice(Math.max(0, start - 160), start);
    const after = code.slice(literal.end, literal.end + 500);
    const blob =
      /(?:(?:var|let|const)\s+([A-Z_a-z$][\w$]*)\s*=\s*)?new\s+Blob\s*\(\s*\[\s*$/.exec(
        before,
      );
    if (
      !blob ||
      !/^\s*,?\s*\]\s*,\s*\{\s*type\s*:\s*['"]application\/javascript['"]/.test(
        after,
      )
    ) {
      continue;
    }

    const workerExpression = blob[1]
      ? new RegExp(
          `new\\s+Worker\\s*\\(\\s*URL\\.createObjectURL\\(\\s*${blob[1].replace(
            /[$]/g,
            '\\$',
          )}\\s*\\)`,
        )
      : /new\s+Worker\s*\(\s*URL\.createObjectURL\s*\(\s*new\s+Blob\s*\(\s*\[\s*$/.test(
            before,
          )
        ? /URL\.createObjectURL/
        : undefined;
    if (workerExpression?.test(blob[1] ? after : before)) {
      sources.push({ start, ...literal });
    }
  }
  return sources;
}

function renderFilename(template: string, content: string): string {
  const hash = createHash('sha256').update(content).digest('hex');
  return template.replace(
    /\[(?:contenthash|hash)(?::(\d+))?\]/g,
    (_match, length) => hash.slice(0, length ? Number(length) : undefined),
  );
}

function joinPublicPath(publicPath: string, filename: string): string {
  return `${publicPath}${publicPath && !publicPath.endsWith('/') ? '/' : ''}${filename}`;
}

/**
 * Extracts large static JavaScript strings passed directly to `Blob` into
 * standalone assets. The original Blob worker is retained with a compact
 * `importScripts` bootstrap, preserving its Blob URL and global scope.
 */
export class ExtractInlineWorkerPlugin {
  private readonly options: Required<
    Pick<ExtractInlineWorkerPluginOptions, 'filename' | 'minSize'>
  > &
    Pick<ExtractInlineWorkerPluginOptions, 'publicPath'>;

  constructor(options: ExtractInlineWorkerPluginOptions = {}) {
    this.options = {
      filename: options.filename ?? 'workers/[contenthash:8].js',
      minSize: Math.max(0, options.minSize ?? 10000),
      publicPath: options.publicPath,
    };
  }

  apply(compiler: Compiler): void {
    const pluginName = 'ExtractInlineWorkerPlugin';
    compiler.hooks.thisCompilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: pluginName,
          stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE + 1,
        },
        () => {
          const configuredPublicPath =
            this.options.publicPath ?? compiler.options.output.publicPath;
          if (
            typeof configuredPublicPath !== 'string' ||
            configuredPublicPath === 'auto'
          ) {
            throw new Error(
              `${pluginName} requires a fixed publicPath option when output.publicPath is not a fixed string`,
            );
          }

          for (const asset of compilation.getAssets()) {
            if (!asset.name.endsWith('.js')) continue;
            const code = asset.source.source().toString();
            const inlineWorkers = findInlineWorkerSources(
              code,
              this.options.minSize,
            );
            if (!inlineWorkers.length) continue;

            const replacement = new compiler.rspack.sources.ReplaceSource(
              asset.source,
              asset.name,
            );
            for (const worker of inlineWorkers) {
              const filename = renderFilename(
                this.options.filename,
                worker.value,
              );
              const existingAsset = compilation.getAsset(filename);
              if (
                existingAsset &&
                existingAsset.source.source().toString() !== worker.value
              ) {
                throw new Error(
                  `${pluginName} generated the same filename for different worker sources: ${filename}`,
                );
              }
              if (!existingAsset) {
                compilation.emitAsset(
                  filename,
                  new compiler.rspack.sources.RawSource(worker.value),
                );
              }
              const bootstrap = `importScripts(${JSON.stringify(
                joinPublicPath(configuredPublicPath, filename),
              )});`;
              replacement.replace(
                worker.start,
                worker.end - 1,
                JSON.stringify(bootstrap),
              );
            }
            compilation.updateAsset(asset.name, replacement);
          }
        },
      );
    });
  }
}
