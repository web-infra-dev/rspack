
it("amd require error callback should be called with the error", function (done) {
	require(['./add'], function () {
		done(new Error('success callback should not be called'));
	}, function (error) {
		expect(error).toBe('some error');
		done();
	});
});
