import createTestCases from '../_helpers/createTestCases.js';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      'lodash-es': [],
    },
  },
  all: {
    usedExports: ['default'],
    expect: {
      'lodash-es': ['uniq'],
    },
  },
});
