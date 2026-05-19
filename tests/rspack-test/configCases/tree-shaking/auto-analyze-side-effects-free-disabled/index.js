import { pureFn } from './decl';

const noUse = pureFn();

it('should not auto analyze no-side-effects functions when disabled', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(2);
});
