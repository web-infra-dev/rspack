const VERSION_PATTERN_REGEXP = /^([\d^=v<>~]|[*xX]$)/;

export function isRequiredVersion(str: string) {
  return VERSION_PATTERN_REGEXP.test(str);
}
