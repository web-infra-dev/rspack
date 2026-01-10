const VERSION_PATTERN_REGEXP = /^([\d^=v<>~]|[*xX]$)/;

export function isRequiredVersion(str: string) {
  return VERSION_PATTERN_REGEXP.test(str);
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
