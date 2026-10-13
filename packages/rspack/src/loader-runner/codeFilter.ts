export interface LoaderCodeFilter {
  include?: (string | RegExp)[];
  exclude: (string | RegExp)[];
}

export interface CodeFilterTextCache {
  initialized: boolean;
  content?: unknown;
  text?: string | null;
}

const decoder = new TextDecoder('utf-8', { fatal: true });

export function createCodeFilterTextCache(): CodeFilterTextCache {
  return { initialized: false };
}

export function invalidateCodeFilterTextCache(cache: CodeFilterTextCache) {
  cache.initialized = false;
  cache.content = undefined;
  cache.text = undefined;
}

function codeAsString(
  content: unknown,
  cache: CodeFilterTextCache,
): string | null {
  if (cache.initialized && cache.content === content) {
    return cache.text ?? null;
  }

  let text: string | null;
  if (typeof content === 'string') {
    text = content;
  } else if (content instanceof Uint8Array) {
    try {
      text = decoder.decode(content);
    } catch {
      text = null;
    }
  } else {
    text = null;
  }

  cache.initialized = true;
  cache.content = content;
  cache.text = text;
  return text;
}

function matchesPatterns(patterns: (string | RegExp)[], code: string) {
  return patterns.some((pattern) => {
    if (typeof pattern === 'string') return code.includes(pattern);
    pattern.lastIndex = 0;
    const matched = pattern.test(code);
    pattern.lastIndex = 0;
    return matched;
  });
}

export function matchesCodeFilter(
  filter: LoaderCodeFilter,
  content: unknown,
  cache: CodeFilterTextCache,
): boolean {
  const code = codeAsString(content, cache);
  if (code === null) return false;
  if (matchesPatterns(filter.exclude, code)) return false;
  return filter.include === undefined || matchesPatterns(filter.include, code);
}
