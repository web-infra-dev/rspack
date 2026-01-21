const { normalizePlaceholder } = require('@rspack/test-tools');

const ANSI = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  cyan: '\x1b[36m',
  gray: '\x1b[90m',
};

expect.extend({
  toBeEquivalentStatsStringWith(received, expected) {
    const { matcherHint, printReceived, diff } = this.utils;

    if (arguments.length < 2 || expected === undefined) {
      return {
        pass: false,
        message: () =>
          matcherHint('.toBeEquivalentStatsString', 'received', 'expected') +
          '\n\nMatcher requires an expected stats string as the second argument.',
      };
    }

    const nReceived = normalizeStatsString(received);
    const nExpected = normalizeStatsString(expected);
    const pass = nReceived === nExpected;

    const message = pass
      ? () =>
          matcherHint('.toBeEquivalentStatsString') +
          '\n\nExpected: not to be equivalent stats string\nReceived (normalized):\n' +
          printReceived(nReceived)
      : () => {
          const rawDiff = diff(nExpected, nReceived);
          const diffOutput = rawDiff ? '\n\nDifference:\n' + colorizeDiff(rawDiff) : '';
          return matcherHint('.toBeEquivalentStatsString', 'received', 'expected') + diffOutput;
        };

    return { pass, message };
  },
});

function shouldUseColor() {
  if (process.env.NO_COLOR !== undefined && process.env.NO_COLOR !== '') {
    return false;
  }
  if (process.env.FORCE_COLOR !== undefined && process.env.FORCE_COLOR !== '0') {
    return true;
  }
  return process.stdout?.isTTY ?? false;
}

function normalizeStatsString(input) {
  if (input == null) return '';

  let out = String(input)
    .replace(/\b\d+(?:\.\d+)?\s*(?:ms|s)\b/gi, 'X ms')
    .replace(/\b[0-9a-f]{7,}\b/gi, '<hash>')
    .replace(/\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\b/g, '<ts>')
    .replace(/\b\d+(?:\.\d+)?\s*(?:B|KB|KiB|MB|MiB|GB|GiB)\b/gi, '<size>');

  // Then use the shared serializer for path normalization
  out = normalizePlaceholder(out);

  return out
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(Boolean)
    .join('\n')
    .replace(/[ \t]+/g, ' ');
}

function colorizeDiff(raw) {
  if (!raw || !shouldUseColor()) return raw;

  return String(raw)
    .split('\n')
    .map(line => {
      if (/^\s*\+/.test(line)) return ANSI.green + line + ANSI.reset;
      if (/^\s*-/.test(line)) return ANSI.red + line + ANSI.reset;
      if (/^\s*@@?/.test(line)) return ANSI.cyan + line + ANSI.reset;
      if (/^(---|(\+\+\+)|diff) /.test(line)) return ANSI.gray + line + ANSI.reset;
      return line;
    })
    .join('\n');
}
