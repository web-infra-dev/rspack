it("should finalize (j|t)sx-in-(j|t)s with internal module type `(j|t)sx`", () => {
	require("./app.js");
	require("./app.ts");
});

it("should work with auto module type finalizations", () => {
	require("./app.tsx");
	require("./app.jsx");
	require("./lib.ts");
	require("./lib.js");
});
