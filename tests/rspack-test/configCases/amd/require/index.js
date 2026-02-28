it("require([...]) should work well", async function () {
	const p = require(['./dep']);
	expect(p).toBeInstanceOf(Promise);

	await p;

	expect(require('./fn').value).toBe(123);
});

it("require([...], function () {}) should work well", () => new Promise(done => {
	require(['./add'], function (add) {
		expect(typeof add).toBe('function');
		expect(add(1, 2)).toBe(3);
		done();
	});
}));
