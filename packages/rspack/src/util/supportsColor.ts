import picocolors from 'picocolors';

/**
 * Whether the current environment supports color output (TTY, FORCE_COLOR, NO_COLOR, etc.).
 * Used as the default for stats.colors when not explicitly set.
 * @see https://github.com/web-infra-dev/rspack/issues/9353
 */
export function isStatsColorSupported(): boolean {
  return picocolors.isColorSupported;
}
