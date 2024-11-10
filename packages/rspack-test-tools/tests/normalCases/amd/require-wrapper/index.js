it('require([], function(require) {}) should work well', function (done) {
	require([], function (require) {
		const add = require('./add');
		expect(typeof add).toBe('function');
		expect(add(1, 2)).toBe(3);
		done();
	});
});
