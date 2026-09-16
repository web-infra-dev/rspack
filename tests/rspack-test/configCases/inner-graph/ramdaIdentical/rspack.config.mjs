import createTestCases from '../_helpers/createTestCases.js';
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
