require('./shared');

it('should execute chunks whose filename templates are functions', async () => {
	const result = await import(/* webpackChunkName: "async" */ './async');
	expect(result.default).toBe('async');
});
