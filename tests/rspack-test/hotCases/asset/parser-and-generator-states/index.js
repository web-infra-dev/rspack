import value from './file';

it("should store and resume asset parser and generator states", async () => {
	expect(value).toBe('string');
	await NEXT_HMR();
	expect(value).toBe('string result');
});

module.hot.accept('./file');