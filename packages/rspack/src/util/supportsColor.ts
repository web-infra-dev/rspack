/**
 * Whether the current environment supports color output (TTY, FORCE_COLOR, NO_COLOR, etc.).
 * Used as the default for stats.colors when not explicitly set.
 * @see https://github.com/web-infra-dev/rspack/issues/9353
 */
export function isStatsColorSupported(): boolean {
  if (typeof process === 'undefined') return false;
  const env = process.env ?? {};
  const argv = process.argv ?? [];
  return (
    !('NO_COLOR' in env || argv.includes('--no-color')) &&
    ('FORCE_COLOR' in env ||
      argv.includes('--color') ||
      process.platform === 'win32' ||
      (process.stdout?.isTTY && env.TERM !== 'dumb') ||
      'CI' in env)
  );
}
