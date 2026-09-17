import createTestCases from '../_helpers/createTestCases.mjs';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      './assert': [],
    },
  },
  fun5: {
    usedExports: ['fun5'],
    expect: {
      './assert': ['deepEqual'],
    },
  },
  fun6: {
    usedExports: ['fun6'],
    expect: {
      './assert': ['equal'],
    },
  },
  all: {
    usedExports: ['fun5', 'fun6'],
    expect: {
      './assert': ['deepEqual', 'equal'],
    },
  },
});
