async function main() {
	const { utilA } = await import(/*webpackChunkName:"utilA"*/ "./utilA.js");
	const { utilB } = await import(/*webpackChunkName:"utilB"*/ "./utilB.js");
	expect(utilA()).toBe("a");
	expect(utilB()).toBe("b");
}

main();

it("should compile and only contain one chunk", () => {
	expect(__STATS__.chunks.length).toBe(1);
});
