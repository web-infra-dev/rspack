import { tokenizer, type Comment, type Token } from 'acorn';
import type { Compiler } from '../Compiler';
import { Compilation } from '../Compilation';

export interface ManglePrivateIdentifiersPluginOptions {
  /**
   * Mutable cache shared by compiler instances that need identical mappings.
   */
  nameCache?: Record<string, string>;
  /**
   * Only mangle identifiers at least this many characters long.
   *
   * @default 8
   */
  minLength?: number;
  /**
   * Selects identifiers eligible for mangling.
   *
   * @default /^_[^_]/
   */
  pattern?: RegExp;
  /** Identifiers that must never be mangled. */
  reserved?: string[];
}

interface JavaScriptAsset {
  name: string;
  tokens: Token[];
}

const IDENTIFIER_IN_TEXT = /[$_\p{ID_Start}][$\p{ID_Continue}]*/gu;
const SHORT_NAME_ALPHABET =
  'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';

function matches(pattern: RegExp, value: string): boolean {
  pattern.lastIndex = 0;
  return pattern.test(value);
}

function renderShortName(index: number): string {
  let current = index;
  let name = '';
  do {
    name = SHORT_NAME_ALPHABET[current % SHORT_NAME_ALPHABET.length] + name;
    current = Math.floor(current / SHORT_NAME_ALPHABET.length) - 1;
  } while (current >= 0);
  return `$${name}`;
}

function addIdentifiersInText(text: string, protectedNames: Set<string>): void {
  for (const match of text.matchAll(IDENTIFIER_IN_TEXT)) {
    protectedNames.add(match[0]);
  }
}

/**
 * Mangles long private-convention identifiers consistently across JavaScript
 * assets. Identifiers visible in strings, templates, regular expressions, or
 * comments are left unchanged so reflective access keeps working.
 */
export class ManglePrivateIdentifiersPlugin {
  private readonly nameCache: Record<string, string>;
  private readonly minLength: number;
  private readonly pattern: RegExp;
  private readonly reserved: Set<string>;

  constructor(options: ManglePrivateIdentifiersPluginOptions = {}) {
    this.nameCache = options.nameCache ?? Object.create(null);
    this.minLength = Math.max(0, options.minLength ?? 8);
    this.pattern = options.pattern ?? /^_[^_]/;
    this.reserved = new Set(options.reserved);
  }

  apply(compiler: Compiler): void {
    const pluginName = 'ManglePrivateIdentifiersPlugin';
    compiler.hooks.thisCompilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: pluginName,
          stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE + 3,
        },
        () => {
          const assets: JavaScriptAsset[] = [];
          const identifierCounts = new Map<string, number>();
          const usedNames = new Set<string>(this.reserved);
          const protectedNames = new Set<string>(this.reserved);

          for (const asset of compilation.getAssets()) {
            if (!asset.name.endsWith('.js')) continue;
            const code = asset.source.source().toString();
            const comments: Comment[] = [];
            const tokens = Array.from(
              tokenizer(code, {
                allowHashBang: true,
                ecmaVersion: 'latest',
                onComment: comments,
              }),
            );

            for (const token of tokens) {
              const value = (token as typeof token & { value?: unknown }).value;
              if (token.type.label === 'name' && typeof value === 'string') {
                usedNames.add(value);
                if (
                  value.length >= this.minLength &&
                  matches(this.pattern, value)
                ) {
                  identifierCounts.set(
                    value,
                    (identifierCounts.get(value) ?? 0) + 1,
                  );
                }
                continue;
              }

              if (
                (token.type.label === 'string' ||
                  token.type.label === 'template') &&
                typeof value === 'string'
              ) {
                addIdentifiersInText(value, protectedNames);
              } else if (
                token.type.label === 'regexp' &&
                value &&
                typeof value === 'object' &&
                'pattern' in value &&
                typeof value.pattern === 'string'
              ) {
                addIdentifiersInText(value.pattern, protectedNames);
              }
            }
            for (const comment of comments) {
              addIdentifiersInText(comment.value, protectedNames);
            }
            assets.push({ name: asset.name, tokens });
          }

          const assignedNames = new Set<string>();
          for (const replacement of Object.values(this.nameCache)) {
            if (assignedNames.has(replacement)) {
              throw new Error(
                `${pluginName} nameCache contains duplicate replacement: ${replacement}`,
              );
            }
            assignedNames.add(replacement);
          }

          let nextName = 0;
          const mappings = new Map<string, string>();
          const candidates = Array.from(identifierCounts, ([name, count]) => ({
            count,
            name,
          })).sort(
            (a, b) =>
              b.count * b.name.length - a.count * a.name.length ||
              a.name.localeCompare(b.name),
          );

          for (const candidate of candidates) {
            if (protectedNames.has(candidate.name)) continue;

            let replacement = this.nameCache[candidate.name];
            if (replacement) {
              if (usedNames.has(replacement)) {
                throw new Error(
                  `${pluginName} cached replacement conflicts with an existing identifier: ${replacement}`,
                );
              }
            } else {
              do {
                replacement = renderShortName(nextName++);
              } while (
                usedNames.has(replacement) ||
                assignedNames.has(replacement)
              );
              if (replacement.length >= candidate.name.length) continue;
              this.nameCache[candidate.name] = replacement;
              assignedNames.add(replacement);
            }
            mappings.set(candidate.name, replacement);
          }

          if (!mappings.size) return;
          for (const asset of assets) {
            const replacement = new compiler.rspack.sources.ReplaceSource(
              compilation.getAsset(asset.name)!.source,
              asset.name,
            );
            let changed = false;
            for (const token of asset.tokens) {
              if (token.type.label !== 'name') continue;
              const value = (token as typeof token & { value?: unknown }).value;
              if (typeof value !== 'string') continue;
              const mangled = mappings.get(value);
              if (!mangled) continue;
              replacement.replace(token.start, token.end - 1, mangled);
              changed = true;
            }
            if (changed) compilation.updateAsset(asset.name, replacement);
          }
        },
      );
    });
  }
}
