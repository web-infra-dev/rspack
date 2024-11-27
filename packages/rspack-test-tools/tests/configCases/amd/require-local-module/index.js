define('add', function () {
	return function add(a, b) {
		return a + b;
	};
});

it('AMD require should support local module', function (done) {
	require(['add'], function (add) {
		expect(typeof add).toBe('function');
		expect(add(1, 2)).toBe(3);
		done();
	});
});
