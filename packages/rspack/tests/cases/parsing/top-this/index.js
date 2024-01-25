import a_extern from "./a";
const a = {
	set b(b) {
		this.c = this._get();
	},
	get a() {
		return this._get();
	},
	get() {
		return this._get();
	},
	_get() {
		return 1;
	}
};

it("should support getter/setter/method in object", () => {
	expect(a_extern).toBe("a");
	expect(a.c).toBeUndefined();
	a.b = 2;
	expect(a.c).toBe(1);
	expect(a.a).toBe(1);
	expect(a.get()).toBe(1);
});
