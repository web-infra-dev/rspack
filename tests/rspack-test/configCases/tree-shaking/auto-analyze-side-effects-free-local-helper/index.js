import { pureUsesHelper } from './decl';

const noUse = pureUsesHelper();

it('should auto analyze same-module helper calls conservatively', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
});
