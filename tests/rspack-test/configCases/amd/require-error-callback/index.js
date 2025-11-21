
it("amd require error callback should be called with the error", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require(['./add'], function () {
		done(new Error('success callback should not be called'));
	}, function (error) {
		expect(error).toBe('some error');
		done();
	});
}));
