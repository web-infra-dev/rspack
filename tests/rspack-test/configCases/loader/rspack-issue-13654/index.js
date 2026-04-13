it("should pass additional data through builtin:swc-loader", () => {
	expect(require("./swc.jsx")).toEqual({
		fromLoader1: "swc.jsx",
		fromLoader2: true
	});
});

it("should pass additional data through builtin:react-refresh-loader", () => {
	expect(require("./react-refresh.jsx")).toEqual({
		fromLoader1: "react-refresh.jsx",
		fromLoader2: true
	});
});

it("should pass additional data through builtin:preact-refresh-loader", () => {
	expect(require("./preact-refresh.jsx")).toEqual({
		fromLoader1: "preact-refresh.jsx",
		fromLoader2: true
	});
});

it("should pass additional data through builtin:lightningcss-loader", () => {
	// Don't pass additional data through builtin:lightningcss-loader
	// This is same behavior with https://github.com/fz6m/lightningcss-loader/blob/a7a9e32317414463b7e593b54d6d7b087b4e7956/src/loader.ts
	// A possible case is:
	//   css-loader <- builtin:lightningcss-loader <- postcss-loader
	//   if we pass additional data then the ast from postcss-loader will be used by css-loader instead of the result of builtin:lightningcss-loader.
	expect(require("./lightning.css")).toEqual({
		fromLoader2: true
	});
});
