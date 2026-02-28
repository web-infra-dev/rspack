it("not panic for require when args is not string ", function () {
	let count = 0;
	try {
		require(1).__expression;
	} catch (_err) {
		count += 1;
	}
	expect(count).toBe(1);
});
