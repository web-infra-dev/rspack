import cache from './loader.mjs!./cache.cjs';
import interceptor from './loader.mjs!./interceptor.cjs';

it('should expose the module cache during build-time execution', () => {
	expect(cache).toEqual({ currentModuleCached: true, exportsShared: true });
});

it('should share execution interceptors with the build-time module loader', () => {
	expect(interceptor).toEqual({ intercepted: true, calls: 1, cached: true });
});
