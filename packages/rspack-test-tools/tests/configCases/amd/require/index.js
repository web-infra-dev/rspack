
it("require([...], function () {}) should work well", function (done) {
	require(['./add'], function (add) {
		expect(typeof add).toBe('function');
		expect(add(1, 2)).toBe(3);
		done();
	});
});
