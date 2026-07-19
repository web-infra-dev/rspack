import { createHash } from 'node:crypto';
import { tokenizer } from 'acorn';
import type { Compiler } from '../Compiler';
import { Compilation } from '../Compilation';

export interface ExtractInlineDataUrlPluginOptions {
  /**
   * Filename template used for extracted data URL contents.
   *
   * @default 'inline-assets/[contenthash:16][ext]'
   */
  filename?: string;
  /**
   * Only extract decoded contents at least this many bytes long.
   *
   * @default 1000
   */
  minSize?: number;
  /**
   * Public path used for extracted assets. By default the plugin uses
   * `output.publicPath` when it is a fixed string.
   */
  publicPath?: string;
}

const DATA_URL =
  /data:([A-Za-z0-9][A-Za-z0-9!#$&^_.+-]*\/[A-Za-z0-9][A-Za-z0-9!#$&^_.+-]*)(?:;[A-Za-z0-9!#$&^_.+-]+=[^;,\s"'()]+)*;base64,([A-Za-z0-9+/]+={0,2})/g;

const MIME_EXTENSIONS: Record<string, string> = {
  'application/wasm': '.wasm',
  'audio/mpeg': '.mp3',
  'font/otf': '.otf',
  'font/ttf': '.ttf',
  'font/woff': '.woff',
  'font/woff2': '.woff2',
  'image/avif': '.avif',
  'image/gif': '.gif',
  'image/jpeg': '.jpg',
  'image/png': '.png',
  'image/svg+xml': '.svg',
  'image/webp': '.webp',
  'video/mp4': '.mp4',
  'video/webm': '.webm',
};

function renderFilename(
  template: string,
  content: Buffer,
  mimeType: string,
): string {
  const hash = createHash('sha256').update(content).digest('hex');
  return template
    .replace(/\[(?:contenthash|hash)(?::(\d+))?\]/g, (_match, length) =>
      hash.slice(0, length ? Number(length) : undefined),
    )
    .replace(/\[ext\]/g, MIME_EXTENSIONS[mimeType] ?? '.bin');
}

function joinPublicPath(publicPath: string, filename: string): string {
  return `${publicPath}${publicPath && !publicPath.endsWith('/') ? '/' : ''}${filename}`;
}

/**
 * Extracts base64 data URLs from JavaScript string literals into standalone
 * assets. This is an opt-in tradeoff that replaces inline resource URLs with
 * public URLs while preserving their decoded contents.
 */
export class ExtractInlineDataUrlPlugin {
  private readonly options: Required<
    Pick<ExtractInlineDataUrlPluginOptions, 'filename' | 'minSize'>
  > &
    Pick<ExtractInlineDataUrlPluginOptions, 'publicPath'>;

  constructor(options: ExtractInlineDataUrlPluginOptions = {}) {
    this.options = {
      filename: options.filename ?? 'inline-assets/[contenthash:16][ext]',
      minSize: Math.max(0, options.minSize ?? 1000),
      publicPath: options.publicPath,
    };
  }

  apply(compiler: Compiler): void {
    const pluginName = 'ExtractInlineDataUrlPlugin';
    compiler.hooks.thisCompilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: pluginName,
          stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE + 2,
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
            const replacements: Array<{
              end: number;
              start: number;
              value: string;
            }> = [];

            for (const token of tokenizer(code, {
              allowHashBang: true,
              ecmaVersion: 'latest',
            })) {
              if (token.type.label !== 'string') continue;
              const value = (token as typeof token & { value: string }).value;
              if (!value.includes('data:')) continue;

              let changed = false;
              const extracted = value.replace(
                DATA_URL,
                (dataUrl, mimeType: string, payload: string) => {
                  const content = Buffer.from(payload, 'base64');
                  if (content.byteLength < this.options.minSize) return dataUrl;

                  const filename = renderFilename(
                    this.options.filename,
                    content,
                    mimeType,
                  );
                  const existingAsset = compilation.getAsset(filename);
                  if (
                    existingAsset &&
                    !Buffer.from(existingAsset.source.source()).equals(content)
                  ) {
                    throw new Error(
                      `${pluginName} generated the same filename for different contents: ${filename}`,
                    );
                  }
                  if (!existingAsset) {
                    compilation.emitAsset(
                      filename,
                      new compiler.rspack.sources.RawSource(content),
                    );
                  }
                  changed = true;
                  return joinPublicPath(configuredPublicPath, filename);
                },
              );
              if (changed) {
                replacements.push({
                  start: token.start,
                  end: token.end,
                  value: JSON.stringify(extracted),
                });
              }
            }

            if (!replacements.length) continue;
            const replacement = new compiler.rspack.sources.ReplaceSource(
              asset.source,
              asset.name,
            );
            for (const item of replacements) {
              replacement.replace(item.start, item.end - 1, item.value);
            }
            compilation.updateAsset(asset.name, replacement);
          }
        },
      );
    });
  }
}
