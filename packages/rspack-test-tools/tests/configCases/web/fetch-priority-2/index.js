it("should set fetchPriority", () => {
	import(/* webpackFetchPriority: "high" */ "./a");
	expect(document.head.children).toHaveLength(3);
	const script1 = document.head.children[1];
	expect(script1.getAttribute("fetchpriority")).toBe("high");

	import(/* webpackFetchPriority: "low" */ "./b");
	expect(document.head.children).toHaveLength(4);
	const script2 = document.head.children[3];
	expect(script2.getAttribute("fetchpriority")).toBe("low");

	import(/* webpackFetchPriority: "low" */ "./c");
	expect(document.head.children).toHaveLength(5);
	const script3 = document.head.children[4];
	expect(script3.getAttribute("fetchpriority")).toBe("low");

	import(/* webpackPrefetch: 20, webpackFetchPriority: "auto" */ "./c");

	import("./d")
	expect(document.head.children).toHaveLength(6);
	const script4 = document.head.children[5];
	expect(script4.getAttribute("fetchpriority")).toBeFalsy();

	import(/* webpackPrefetch: -20 */ "./d3");
	expect(document.head.children).toHaveLength(7);
	const script5 = document.head.children[6];
	expect(script5.getAttribute("fetchpriority")).toBeFalsy();

	const condition = true;

	if (!condition) {
		import(/* webpackFetchPriority: "high", webpackChunkName: "one" */ "./e");
		expect(document.head.children).toHaveLength(8);
		const script6 = document.head.children[7];
		expect(script6.getAttribute("fetchpriority")).toBe("high");
	} else {
		import(/* webpackFetchPriority: "low", webpackChunkName: "two" */ "./e");
		expect(document.head.children).toHaveLength(8);
		const script6 = document.head.children[7];
		expect(script6.getAttribute("fetchpriority")).toBe("low");
	}
});
