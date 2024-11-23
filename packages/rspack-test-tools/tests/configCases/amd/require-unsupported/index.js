it('\'require(…, …)\' should throw an error if cannot be statically analysed', function () {
	expect(function () {
		require({}, function () { });
	}).toThrow();
});
