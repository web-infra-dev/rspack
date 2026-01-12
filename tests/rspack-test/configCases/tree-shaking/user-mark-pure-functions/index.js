import { pureFn } from './decl'; // this should have no side effects at all

const noUse = pureFn()
it('should have no side effects for all', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
})
