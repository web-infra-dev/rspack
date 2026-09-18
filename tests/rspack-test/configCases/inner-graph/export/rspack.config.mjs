import createTestCases from '../_helpers/createTestCases.mjs';
export default createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      './components/Button': [],
      './components/ButtonGroup': [],
      './theme': [],
    },
  },
  all: {
    usedExports: ['default', 'ButtonGroup', 'themeNamespace'],
    expect: {
      './components/Button': ['default'],
      './components/ButtonGroup': ['default'],
      './theme': ['themeNamespace'],
    },
  },
});
