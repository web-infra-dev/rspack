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
	expect(require("./lightning.css")).toEqual({
		fromLoader1: "lightning.css",
		fromLoader2: true
	});
});
