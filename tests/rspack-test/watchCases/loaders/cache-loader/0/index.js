import trigger from './trigger';
import value from './value';

it('should reuse the cached loader result across compilations', () => {
  expect(trigger).toBe(WATCH_STEP);
  expect(value).toBe(1);
});
