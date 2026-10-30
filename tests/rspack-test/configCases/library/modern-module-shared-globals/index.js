import './globals.js';
import shared from './shared.cjs';

it('should preserve free identifiers in shared CommonJS factories', async () => {
  const other = await import(/* webpackIgnore: true */ OTHER_BUNDLE);
  expect(shared).toEqual(['global-modules', 'global-register']);
  expect(other.shared).toEqual(['global-modules', 'global-register']);
});

it('should preserve free identifiers in shared async CommonJS factories', async () => {
  const other = await import(/* webpackIgnore: true */ OTHER_BUNDLE);
  const first = await import('./async.cjs');
  const second = await other.load();
  expect(first.default()).toEqual(['global-modules', 'global-register']);
  expect(second.default()).toEqual(['global-modules', 'global-register']);
});
