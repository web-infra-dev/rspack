export const localPatternTag = 'local-rsc-pattern';

export function describeModuleType(value: unknown) {
  return typeof value;
}

export function composeExposeMessage(parts: Array<string | number | boolean>) {
  return parts.join(':');
}
