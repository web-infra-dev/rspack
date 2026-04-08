import { pureFn } from './decl';

let sideEffectCount = 0;

function sideEffect() {
  sideEffectCount += 1;
  return sideEffectCount;
}

pureFn(sideEffect());

it('should keep side effects in call arguments', () => {
  expect(sideEffectCount).toBe(1);
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(2);
});
