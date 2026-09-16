import createTestCases from '../_helpers/createTestCases.js';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      './assert': ['deepEqual'],
    },
  },
});
