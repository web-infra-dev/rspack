import createTestCases from '../_helpers/createTestCases.mjs';
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
