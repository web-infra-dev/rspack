import shared from './shared.cjs';
import child from './child.cjs';
import cacheOnly from './cache-only.cjs';

it('should read and invalidate the executing runtime cache', async () => {
  const other = await import(/* webpackIgnore: true */ OTHER_BUNDLE);
  expect(shared.cache).toBe(cacheOnly);
  expect(other.shared.cache).toBe(other.cacheOnly);
  expect(shared.child).toBe(child);
  expect(other.shared.child).toBe(other.child);
  expect(shared.cached()).toBe(child);
  expect(other.shared.cached()).toBe(other.child);
  expect(shared.cache === other.shared.cache).toBe(SINGLE_RUNTIME);

  other.shared.setCache(other.shared.cache);
  expect(shared.cached()).toBe(child);
  expect(other.shared.cached()).toBe(other.child);

  const reloaded = other.shared.reload();
  expect(reloaded).not.toBe(other.child);
  expect(other.shared.cached()).toBe(reloaded);
  expect(shared.cached()).toBe(SINGLE_RUNTIME ? reloaded : child);
});
