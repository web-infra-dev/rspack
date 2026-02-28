var x = require("./module");

it("should allow to hot replace modules in a ConcatenatedModule", async () => {
	expect(x).toEqual(nsObj({
		default: "ok1"
	}));
	await NEXT_HMR();
	x = require("./module");
	expect(x).toEqual(nsObj({
		default: "ok2"
	}));
});

module.hot.accept("./module");
