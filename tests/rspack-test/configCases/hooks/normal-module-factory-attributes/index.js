import fs from 'fs';
import path from 'path';
import plain from './plain.js';
import withAttributes from './with-attributes.js' with {
	type: 'custom',
	flavor: 'spicy'
};

const readSeen = () =>
	JSON.parse(
		fs.readFileSync(path.resolve(__dirname, './attributes.json'), 'utf-8')
	);

const hooks = ['beforeResolve', 'factorize', 'resolve', 'afterResolve'];

it("should expose a static import's attributes to every NormalModuleFactory hook", () => {
	expect(withAttributes).toBe('with-attributes');
	const seen = readSeen();
	for (const hook of hooks) {
		expect(seen['./with-attributes.js'][hook]).toEqual({
			type: 'custom',
			flavor: 'spicy'
		});
	}
});

it("should expose a dynamic import's attributes to every NormalModuleFactory hook", async () => {
	const dynamic = await import('./dynamic.js', { with: { level: '2' } });
	expect(dynamic.default).toBe('dynamic');
	const seen = readSeen();
	for (const hook of hooks) {
		expect(seen['./dynamic.js'][hook]).toEqual({ level: '2' });
	}
});

it('should leave attributes undefined for an import without attributes', () => {
	expect(plain).toBe('plain');
	const seen = readSeen();
	for (const hook of hooks) {
		expect(seen['./plain.js'][hook]).toBe('<undefined>');
	}
});
