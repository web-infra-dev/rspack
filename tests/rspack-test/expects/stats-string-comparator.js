const { normalizePlaceholder } = require('@rspack/test-tools');

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
          const diffOutput = diff(nExpected, nReceived);
          return matcherHint('.toBeEquivalentStatsString', 'received', 'expected') + diffOutput;
        };

    return { pass, message };
  },
});

function normalizeStatsString(input) {
  if (input == null) return '';

  let out = String(input)
    .replace(/\b\d+(?:\.\d+)?\s*(?:ms|s)\b/gi, 'X ms')
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
