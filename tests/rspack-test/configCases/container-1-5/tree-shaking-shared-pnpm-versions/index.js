import { value } from 'ui-lib';
import { nestedValue } from 'dep';

it('should generate tree shaking shared fallbacks for every resolved version', () => {
  expect(value).toEqual('direct-1');
  expect(nestedValue).toEqual('nested-2');

  const fallbacks = __webpack_require__.federation.sharedFallback['ui-lib'];
  expect(fallbacks.map(([, version]) => version).sort()).toEqual([
    '1.0.0',
    '2.0.0',
  ]);
});
