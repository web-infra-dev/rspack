import "./a.css";
import "./b.css";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should load a chunk with css", () => {
	const linkStart = document.getElementsByTagName("link").length;
	const scriptStart = document.getElementsByTagName("script").length;
	const promise = import("./chunk");

	const links = document.getElementsByTagName("link").slice(linkStart);
	const scripts = document.getElementsByTagName("script").slice(scriptStart);

	expect(links.length).toBe(1);
	expect(scripts.length).toBe(1);
	links[0].onload({ type: "load" });
	__non_webpack_require__(
		scripts[0].src.replace("https://test.cases/path", ".")
	);

	const css = fs
		.readFileSync(
			path.resolve(
				__dirname,
				links[0].href.replace("https://test.cases/path", ".")
			),
			"utf-8"
		)
		.trim();
	// CHANGE: we use rspack-test-tools to run webpack watchCases for incremental, its inline
	// snapshot result is different with webpack tester caused by different SnapshotSerializer
	// (see packages/rspack-test-tools/src/helper/setup-expect.ts)
	const snapshot = `\
.chunk {
	color: red;
}`
	expect(css).toEqual(snapshot);

	return promise;
});

it("should generate correct css", () => {
	const css = fs
		.readFileSync(path.resolve(__dirname, "main.css"), "utf-8")
		.trim();
	// CHANGE: we use rspack-test-tools to run webpack watchCases for incremental, its inline
	// snapshot result is different with webpack tester caused by different SnapshotSerializer
	// (see packages/rspack-test-tools/src/helper/setup-expect.ts)
	const snapshot = `\
.dependency {
	color: ${WATCH_STEP === "1" ? "red" : "green"};
}

.a {
	color: red;
}

.b {
	color: ${WATCH_STEP === "1" ? "red" : "green"};
}`
	expect(css).toEqual(snapshot);
});

if (WATCH_STEP !== "1") {
	it("should not emit javascript again", () => {
		expect(
			__STATS__.assets.filter(a => a.name.endsWith(".js"))
		).not.toContainEqual(
			expect.objectContaining({
				cached: false
			})
		);
	});
}
