import { createHash } from '../util/createHash';

const VERSION_PATTERN_REGEXP = /^([\d^=v<>~]|[*xX]$)/;

export function isRequiredVersion(str: string) {
  return VERSION_PATTERN_REGEXP.test(str);
}

export function encodeName(...parts: string[]): string {
  return `mf_${createHash('xxhash64').update(JSON.stringify(parts), 'utf8').digest('hex')}`;
}
