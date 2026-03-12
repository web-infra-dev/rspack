export { aValue, aHelper } from './a';
export { bValue, bHelper } from './b';
export { cValue, cHelper } from './c';
export { dValue, dHelper } from './d';
export { eValue, eHelper } from './e';

it('should deconflict many modules with same local variable names', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.aValue).toBe('a');
	expect(mod.aHelper).toBe('A');
	expect(mod.bValue).toBe('b');
	expect(mod.bHelper).toBe('B');
	expect(mod.cValue).toBe('c');
	expect(mod.cHelper).toBe('C');
	expect(mod.dValue).toBe('d');
	expect(mod.dHelper).toBe('D');
	expect(mod.eValue).toBe('e');
	expect(mod.eHelper).toBe('E');
});
