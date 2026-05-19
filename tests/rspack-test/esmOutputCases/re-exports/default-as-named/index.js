export { renamedDefault, named } from './middle.js';

it('should re-export default as named through a chain', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.renamedDefault).toBe(42);
	expect(mod.named).toBe('hello');
});
