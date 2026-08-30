'use strict';

const createTestCases = require('../_helpers/createTestCases');

module.exports = createTestCases({
  nothing: {
    usedExports: [],
    expect: {
      any: ['fn', 'im1'],
    },
  },
  pureTpl: {
    usedExports: ['pureTpl'],
    expect: {
      any: ['fn', 'im1'],
    },
  },
  unknownTpl: {
    usedExports: ['unknownTpl'],
    expect: {
      any: ['fn', 'im1'],
    },
  },
  impureTpl: {
    usedExports: ['impureTpl'],
    expect: {
      any: ['fn', 'im1'],
    },
  },
});
