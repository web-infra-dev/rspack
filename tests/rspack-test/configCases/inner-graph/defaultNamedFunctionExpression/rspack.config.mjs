import createTestCases from '../_helpers/createTestCases.mjs';
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
