const VERSION_PATTERN_REGEXP = /^([\d^=v<>~]|[*xX]$)/;

export function isRequiredVersion(str: string) {
  return VERSION_PATTERN_REGEXP.test(str);
}

export function resolveShareRequest(
  request: string | undefined,
  fallback: string,
) {
  return request || fallback;
}

export function resolveShareKey(
  shareKey: string | undefined,
  fallback: string,
) {
  return shareKey || fallback;
}

export function resolveShareScope(
  shareScope?: string | string[],
  fallbackShareScope?: string | string[],
) {
  return shareScope || fallbackShareScope || 'default';
}

export const encodeName = function (
  name: string,
  prefix = '',
  withExt = false,
): string {
  const ext = withExt ? '.js' : '';
  return `${prefix}${name
    .replace(/@/g, 'scope_')
    .replace(/-/g, '_')
    .replace(/\//g, '__')
    .replace(/\./g, '')}${ext}`;
};
