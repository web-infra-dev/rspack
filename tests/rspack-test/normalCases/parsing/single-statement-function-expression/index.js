function helper(value) {
	return value;
}

// Function expressions are valid operands in any expression position, so
// these unbraced if/while bodies must parse.
if (true)
	helper(function* () {
		yield 1;
	});

let asyncFunction;
if (true)
	asyncFunction = helper(async function () {
		await 1;
		return "ok";
	});

function* generator() {
	if (true)
		yield [1].map(value =>
			helper(function* () {
				yield value;
			})
		);
}

while (false)
	helper(function* () {
		yield 2;
	});

it("should parse function expressions in single-statement contexts", async () => {
	const yielded = [...generator()];
	expect(yielded.length).toBe(1);
	expect(yielded[0][0]().next().value).toBe(1);
	expect(await asyncFunction()).toBe("ok");
});
