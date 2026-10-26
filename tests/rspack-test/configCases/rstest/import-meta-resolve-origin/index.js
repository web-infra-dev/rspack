import { getResolve, resolveSibling } from './nested/resolver';
import {
	getResolveType,
	resolveConditional,
	resolveTarget,
} from './bundler-resolve';

const resolvers = new Map();
function factory(origin) {
	if (!resolvers.has(origin)) {
		resolvers.set(origin, (specifier, parent = origin) => ({
			specifier,
			parent,
		}));
	}
	return resolvers.get(origin);
}
globalThis.__rstest_import_meta_resolve__ = factory;
globalThis.__custom_import_meta_resolve__ = factory;

it('respects importMetaResolve enabled globally or by a module rule', () => {
	expect(getResolveType()).toBe('function');
	expect(resolveTarget()).toBe(require.resolve('./target'));
	expect(resolveConditional(true)).toBe(require.resolve('./target'));
	expect(resolveConditional(false)).toBe(require.resolve('./nested/resolver'));
});

it('preserves source origins for calls and resolver values', () => {
	expect(typeof import.meta.resolve).toBe('function');
	const resolve = import.meta.resolve;
	expect(resolve).toBe(import.meta.resolve);
	const expected = {
		specifier: './target',
		parent: expect.stringMatching(
			/[\\/]import-meta-resolve-origin[\\/]index\.js$/,
		),
	};
	expect(import.meta.resolve('./target')).toEqual(expected);
	const specifier = './target';
	expect(resolve(specifier)).toEqual(expected);
	expect(import.meta['resolve'](specifier)).toEqual(expected);
	expect(import.meta.resolve?.(specifier)).toEqual(expected);
	const { resolve: destructured } = import.meta;
	expect(destructured).toBe(resolve);
	expect(import.meta.resolve('./target', 'custom-parent')).toEqual({
		specifier: './target',
		parent: 'custom-parent',
	});
	expect(getResolve()).toBe(getResolve());
	expect(getResolve()).not.toBe(resolve);
	expect(resolveSibling(specifier)).toEqual({
		specifier,
		parent: expect.stringMatching(/[\\/]nested[\\/]resolver\.js$/),
	});
});

it('continues walking nested call arguments', () => {
	expect(import.meta.resolve(require('./target').name).specifier).toBe(
		'nested',
	);
});
