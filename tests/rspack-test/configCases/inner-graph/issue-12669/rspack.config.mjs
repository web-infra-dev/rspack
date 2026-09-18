import createTestCases from '../_helpers/createTestCases.mjs';
// TODO: figure out why some exports are missing
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      '@effect-ts/tracing-utils': [],
      '@effect-ts/system/Option': [
        'extend',
        'flatten',
        'getLeft',
        'getRight',
        'isNone',
        'map',
        'map_',
        'none',
        // "some",
        'zip',
      ],
      '../Associative': [],
      '../Either': [
        // "left",
        // "right"
      ],
      '../Function': [],
      '../Identity': [],
      '../Ord': [],
      '../Prelude': [
        // "implementCompactF",
        // "implementForEachF",
        // "implementSeparateF",
        'instance',
        'matchers',
        // "orElseF",
        // "structF",
        // "succeedF",
        // "tupleF"
      ],
    },
  },
  if: {
    usedExports: ['if'],
    expect: {
      '@effect-ts/tracing-utils': [],
      '@effect-ts/system/Option': [
        'extend',
        'flatten',
        'getLeft',
        'getRight',
        'isNone',
        'map',
        'map_',
        'none',
        // "some",
        'zip',
      ],
      '../Associative': [],
      '../Either': [
        // "left", "right"
      ],
      '../Function': [],
      '../Identity': [],
      '../Ord': [],
      '../Prelude': [
        // "implementCompactF",
        // "implementForEachF",
        // "implementSeparateF",
        'instance',
        'matchers',
        // "orElseF",
        // "structF",
        // "succeedF",
        // "tupleF",
        'conditionalF',
      ],
    },
  },
});
