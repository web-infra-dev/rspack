import stylesheet from "./stylesheet.css.js";

it("should be able to use build-time code with HMR", async () => {
	expect(stylesheet).toBe(
		'body { background: url("https://test.cases/path/assets/file.png"); color: #f00; }'
	);
	await NEXT_HMR();
	expect(stylesheet).toBe(
		'body { background: url("https://test.cases/path/assets/file.png"); color: #0f0; }'
	);
	await NEXT_HMR();
	expect(stylesheet).toBe(
		'body { background: url("https://test.cases/path/assets/file.jpg"); color: #00f; }'
	);
});

import.meta.webpackHot.accept("./stylesheet.css.js");
