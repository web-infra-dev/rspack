import x from "./#";

it("resolver should work when path with symbol", () => {
	expect(x["@@iterator"]).toBe("@@iterator");
	expect(x.g).toBe("g");
	expect(x.a).toBe("_a-b-c");
	expect(x.d).toBe("d-e_f");
});
