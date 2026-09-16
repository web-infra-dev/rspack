import createTestCases from '../_helpers/createTestCases.js';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      any: [],
    },
  },
  default: {
    usedExports: ['default'],
    expect: {
      any: ['fun1'],
    },
  },
});
