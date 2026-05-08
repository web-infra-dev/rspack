const eagerModules = import.meta.glob('./dir/*.js', { eager: true });

it("should import all modules via glob (eager)", () => {
	expect(eagerModules['./dir/foo.js']).toBe("foo");
	expect(eagerModules['./dir/bar.js']).toBe("bar");
	expect(eagerModules['./dir/baz.js']).toBe("baz");
	expect(Object.keys(eagerModules).length).toBe(3);
});
