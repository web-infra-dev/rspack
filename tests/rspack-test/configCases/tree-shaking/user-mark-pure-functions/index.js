import { pureFn } from './decl'; // this should have no side effects at all


it('should have no side effects for all', () => {
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(1);
})
