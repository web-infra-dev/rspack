import createTestCases from '../_helpers/createTestCases.mjs';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      './assert': ['equal'],
    },
  },
  myFunction: {
    usedExports: ['myFunction'],
    expect: {
      './assert': ['deepEqual', 'equal'],
    },
  },
});
