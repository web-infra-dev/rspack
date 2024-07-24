it("should set fetchPriority", () => {
	// Single Chunk
	import(/* webpackFetchPriority: "high" */ "./a.js");
	expect(document.head.children).toHaveLength(1);
	const script1 = document.head.children[0];
	expect(script1.getAttribute("fetchpriority")).toBe("high");

	// Multiple Chunks
	import(/* webpackFetchPriority: "high" */ "./b.js");
	import(/* webpackFetchPriority: "high" */ "./b2.js");
	expect(document.head.children).toHaveLength(4);
	const script2 = document.head.children[1];
	const script3 = document.head.children[2];
	const script4 = document.head.children[3];
	expect(script2.getAttribute("fetchpriority")).toBe("high");
	expect(script3.getAttribute("fetchpriority")).toBe("high");
	expect(script4.getAttribute("fetchpriority")).toBe("high");

	// Single Chunk, low
	import(/* webpackFetchPriority: "low" */ "./c.js");
	expect(document.head.children).toHaveLength(5);
	const script5 = document.head.children[4];
	expect(script5.getAttribute("fetchpriority")).toBe("low");

	// Single Chunk, auto
	import(/* webpackFetchPriority: "auto" */ "./d.js");
	expect(document.head.children).toHaveLength(6);
	const script6 = document.head.children[5];
	expect(script6.getAttribute("fetchpriority")).toBe("auto");

	// No fetch priority
	import("./e.js");
	expect(document.head.children).toHaveLength(7);
	const script7 = document.head.children[6];
	expect(script7.getAttribute("fetchpriority")).toBeFalsy();

	// Webpack context
	const loader = import.meta.webpackContext("./dir", {
		mode: "lazy",
		fetchPriority: "high"
	});
	loader("./a");
	expect(document.head.children).toHaveLength(8);
	const script8 = document.head.children[7];
	expect(script8.getAttribute("fetchpriority")).toBeFalsy();

	import(/* webpackFetchPriority: "auto" */ "./g.js");
	expect(document.head.children).toHaveLength(9);
	const script9 = document.head.children[8];
	expect(script9.getAttribute("fetchpriority")).toBe("auto");

	import(/* webpackFetchPriority: "unknown" */ "./h.js");
	expect(document.head.children).toHaveLength(10);
	const script10 = document.head.children[9];
	expect(script10.getAttribute("fetchpriority")).toBeFalsy();

	import(/* webpackFetchPriority: "high" */ "./i.js");
	import(/* webpackFetchPriority: "low" */ "./i.js");
	expect(document.head.children).toHaveLength(11);
	const script11 = document.head.children[10];
	expect(script11.getAttribute("fetchpriority")).toBe("high");

	import(/* webpackFetchPriority: "low" */ "./j.js");
	import(/* webpackFetchPriority: "high" */ "./j.js");
	expect(document.head.children).toHaveLength(12);
	const script12 = document.head.children[11];

	expect(script12.getAttribute("fetchpriority")).toBe("low");
	import(/* webpackFetchPriority: "low" */ "./k.js");
	import("./e.js");
	import(/* webpackFetchPriority: "high" */ "./k.js");
	expect(document.head.children).toHaveLength(13);
	const script13 = document.head.children[12];
	expect(script13.getAttribute("fetchpriority")).toBe("low");

	import(/* webpackFetchPriority: "high" */ "./style.css");
	expect(document.head.children).toHaveLength(15);
	const link1 = document.head.children[13];
	expect(link1.getAttribute("fetchpriority")).toBe("high");

	import("./style-1.css");
	expect(document.head.children).toHaveLength(17);
	const link2 = document.head.children[15];
	expect(link2.getAttribute("fetchpriority")).toBeFalsy();

	import(/* webpackFetchPriority: "low" */ "./style-2.css");
	expect(document.head.children).toHaveLength(19);
	const link3 = document.head.children[17];
	expect(link3.getAttribute("fetchpriority")).toBe("low");
});
