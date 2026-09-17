import createTestCases from '../_helpers/createTestCases.js';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      any: ['fun2', 'var1'],
    },
  },
  all: {
    usedExports: ['default'],
    expect: {
      any: ['fun2', 'var1'],
    },
  },
});
