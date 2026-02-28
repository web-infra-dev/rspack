export function encodeVersion(version: string) {
  const [major, minor = 0, patch = 0] = version
    .split('-')[0]
    .split('.')
    .map((v) => parseInt(v, 10));

  if (Number.isNaN(major) || Number.isNaN(minor) || Number.isNaN(patch)) {
    return null;
  }

  return (major << 16) | (minor << 8) | patch;
}

export function decodeVersion(n: number) {
  const major = (n >> 16) & 0xff;
  const minor = (n >> 8) & 0xff;
  const patch = n & 0xff;
  return `${major}.${minor}.${patch}`;
}
