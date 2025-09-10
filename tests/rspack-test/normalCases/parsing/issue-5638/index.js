it("require computed member expression of exports", () => {
	expect(require("./computed")).toEqual({
		ArrayBuffer: true,
		DataView: true
	});
});

it("require computed require member expression of exports", () => {
	expect(require("./computed-require")).toEqual({
		ArrayBuffer: true,
		DataView: true
	});
});

it("require computed member expression of exports.xxx", () => {
	expect(require("./computed-nested")).toEqual({
		a: {
			ArrayBuffer: true,
			DataView: true
		}
	});
});

it("require computed member expression of exports[xxx].xxx", () => {
	expect(require("./computed-inner")).toEqual({
		ArrayBuffer: {
			a: true
		},
		DataView: {
			a: true
		}
	});
});
