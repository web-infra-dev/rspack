import createTestCases from '../_helpers/createTestCases.js';
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
