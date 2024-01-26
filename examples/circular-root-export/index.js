it("should hoist exports in a concatenated module", () => {
	return import("./root-ref").then(m => {
		m.test();
	});
});

if (Math.random() < 0) import("./external-ref");


function expect(v) {
	return {
		toBe(vv) {
			if (v == v) {
				return
			}
			throw new Error(`left ${v}, right: ${vv}`)
		},
		toThrow(fn) {
			try {
				fn()
			} catch {

			}
			throw new Error("should throw")
		}
	}
}
function it(name, fn) {
	fn()
}


global.expect = expect;
global.it = it;
