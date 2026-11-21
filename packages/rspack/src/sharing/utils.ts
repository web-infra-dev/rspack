import { createHash } from '../util/createHash';

const VERSION_PATTERN_REGEXP = /^([\d^=v<>~]|[*xX]$)/;

export function validateLayer(
  layer: string | undefined,
  plugin: string,
  option = 'layer',
) {
  if (layer === '') {
    throw new Error(`[${plugin}] ${option} must be a non-empty string`);
  }
}

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

export function encodeName(...parts: string[]): string {
  return `mf_${createHash('xxhash64').update(JSON.stringify(parts), 'utf8').digest('hex')}`;
}
