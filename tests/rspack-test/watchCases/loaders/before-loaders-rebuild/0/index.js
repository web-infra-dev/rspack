import { tag, value } from './dep';

it('should rebuild the module on every step', () => {
  expect(value).toBe(`build${WATCH_STEP}`);
});

it('should apply the loader the beforeLoaders tap added, on every build', () => {
  // `tag` only exists because the tap pushed tag-loader onto the list. If a
  // rebuild dropped the tap's change, this export would be missing.
  expect(tag).toBe('tagged-by-beforeLoaders');
});
