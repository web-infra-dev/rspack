const f = function (require) {
	if (require) {
		throw Error('unreachable')
	} else if (require.resolve) {
		throw Error('unreachable')
	} else if (require.resolveWeak) {
		throw Error('unreachable')
	}
	return 1
}


it("should return value correctly", function () {
	expect(f(false)).toBe(1)
});
