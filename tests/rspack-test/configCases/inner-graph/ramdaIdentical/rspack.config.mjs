import createTestCases from '../_helpers/createTestCases.mjs';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      './internal/_curry2': [],
    },
  },
  all: {
    usedExports: ['default'],
    expect: {
      './internal/_curry2': ['default'],
    },
  },
});
