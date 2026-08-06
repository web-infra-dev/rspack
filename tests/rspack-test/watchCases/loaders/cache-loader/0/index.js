import builtinValue from './builtin-value';
import trigger from './trigger';
import value from './value';

it('should reuse the cached loader result across compilations', () => {
  const step = Number(WATCH_STEP);
  expect(trigger).toBe(step);
  expect(value).toBe(step === 2 ? 2 : 1);
  expect(builtinValue).toBe(step === 2 ? 2 : 1);
});
