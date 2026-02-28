it('require([], function(require) {}) should work well', () => new Promise(done => {
	require(['./hello'], function (hello, require) {
		expect(typeof hello).toBe('function');
		expect(hello('world')).toBe('hello, world');

		const add = require('./add');
		expect(typeof add).toBe('function');
		expect(add(1, 2)).toBe(3);
		done();
	});
}));
