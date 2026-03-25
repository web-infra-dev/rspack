import './setup';
import './decl';

it('should keep calls for undefined sideEffectsFree hints', () => {
  expect(globalThis.sideEffectCount).toBe(1);
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(3);
});
