const { expectWarningFactory } = require('@rspack/test-tools/helper/util/expectWarningFactory');
const expectWarning = expectWarningFactory();

it('should distinguish singleton consumes from strict version consumes', () => {
  __webpack_share_scopes__.default = {
    shared: {
      '1.0.0': { get: () => () => 'shared@1.0.0' },
      '2.0.0': { get: () => () => 'shared@2.0.0' },
    },
  };

  // The highest singleton version is selected even when it does not satisfy the range.
  expect(require('singleton')).toBe('shared@2.0.0');
  expectWarning(/Unsatisfied version 2\.0\.0.*shared/);
  expect(require('strict')).toBe('shared@1.0.0');
});
