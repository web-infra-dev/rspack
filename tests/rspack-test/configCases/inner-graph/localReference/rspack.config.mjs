import createTestCases from '../_helpers/createTestCases.mjs';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      './assert': ['deepEqual'],
    },
  },
});
