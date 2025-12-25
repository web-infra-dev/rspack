import { pureFn } from './decl'; // this should have no side effects at all

// no side effects for this call, so it should be removed
const _ = pureFn();

it('should have no side effects for all', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
})
