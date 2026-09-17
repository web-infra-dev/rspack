import createTestCases from '../_helpers/createTestCases.js';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      any: [],
    },
  },
  exp1: {
    usedExports: ['default'],
    expect: {
      any: ['fun1'],
    },
  },
});
