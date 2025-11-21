import { a, b } from "./a"

it("should generate correct import specifier", () => {
	expect(a).toBe("aaa");
	expect(b).toBe("bbb");
})

it("should generate correct code for var inside class blocks", function () {
	class A {
		constructor() {
			var a;
			a = 1;
			expect(a).toBe(1)
		}
		a() {
			var a;
			a = 1;
			expect(a).toBe(1);
			this.#a();
			expect(this.#b).toBe(2);
			return true;
		}
		#a() {
			var a;
			a = 1;
			expect(a).toBe(1)
		}
		#b = 2;
	}
	expect(a).toBe(undefined);
	var a = new A();
	expect(a.a()).toBe(true);
});
