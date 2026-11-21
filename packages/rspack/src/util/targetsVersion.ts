export function encodeVersion(version: string) {
  const [major, minor = 0, patch = 0] = version
    .split('-')[0]
    .split('.')
    .map((v) => parseInt(v, 10));

  if (
    Number.isNaN(major) ||
    Number.isNaN(minor) ||
    Number.isNaN(patch) ||
    major < 0 ||
    major > 0xffff ||
    minor < 0 ||
    patch < 0
  ) {
    return null;
  }

  return (
    ((major << 16) | (Math.min(minor, 0xff) << 8) | Math.min(patch, 0xff)) >>> 0
  );
}

export function decodeVersion(n: number) {
  const major = n >>> 16;
  const minor = (n >> 8) & 0xff;
  const patch = n & 0xff;
  return `${major}.${minor}.${patch}`;
}
