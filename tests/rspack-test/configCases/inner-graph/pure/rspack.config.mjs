import createTestCases from '../_helpers/createTestCases.mjs';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      './internal/_curry1': [],
      './curryN': [],
      './reduce': [],
      './max': [],
      './pluck': [],
    },
  },
  all: {
    usedExports: ['default'],
    expect: {
      './internal/_curry1': ['default'],
      './curryN': ['default'],
      './reduce': ['default'],
      './max': ['default'],
      './pluck': ['default'],
    },
  },
});
