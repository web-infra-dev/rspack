import shared from './shared.cjs';
import counter from './counter.cjs';

it('should keep nested CommonJS module state per runtime', async () => {
  const other = await import(/* webpackIgnore: true */ OTHER_BUNDLE);

  expect(shared.counter).toBe(counter);
  expect(other.shared.counter).toBe(other.counter);
  expect(shared.counter.get()).toBe(0);
  expect(other.shared.counter.get()).toBe(0);

  shared.counter.inc();

  expect(shared.counter.get()).toBe(1);
  if (SINGLE_RUNTIME) {
    expect(other.shared.counter).toBe(shared.counter);
    expect(other.shared.counter.get()).toBe(1);
  } else {
    expect(other.shared.counter).not.toBe(shared.counter);
    expect(other.shared.counter.get()).toBe(0);
  }
});

it('should use the calling runtime for dynamic imports in shared factories', async () => {
  const other = await import(/* webpackIgnore: true */ OTHER_BUNDLE);
  const first = await shared.load();
  const second = await other.shared.load();
  expect(first.default).toBe(counter);
  expect(second.default).toBe(other.counter);
});
