import shared from './shared.cjs';
import { __rspack_shared_modules__, __rspack_register_modules__ } from './marker.js';

it('should register shared CommonJS factories in every entry runtime', async () => {
  const other = await import(/* webpackIgnore: true */ OTHER_BUNDLE);
  expect(shared.value).toBe(42);
  expect(other.shared.value).toBe(42);
  expect(other.shared === shared).toBe(SINGLE_RUNTIME);
  expect(__rspack_shared_modules__).toBe('modules');
  expect(__rspack_register_modules__).toBe('register');
  expect(other.markers).toEqual(['modules', 'register']);
});

it('should register shared async factories in every entry runtime', async () => {
  const other = await import(/* webpackIgnore: true */ OTHER_BUNDLE);
  const first = await import('./async.cjs');
  const second = await other.load();
  expect(first.default.value).toBe(42);
  expect(second.default.value).toBe(42);
  expect(first.default === second.default).toBe(SINGLE_RUNTIME);
});
