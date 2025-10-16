function runWithThis(obj, fn) {
	fn.call(obj);
}

it("should bind this context on require callback", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require("./file");
	runWithThis({ok: true}, function() {
		require([], function() {
			try {
				expect(require("./file")).toBe("file");
				expect(this).toEqual({ok: true});
				done();
			} catch(e) { done(e); }
		}.bind(this));
	});
}));

it("should bind this context on require callback (loaded)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	runWithThis({ok: true}, function() {
		require(["./load.js"], function(load) {
			try {
				expect(require("./file")).toBe("file");
				expect(load).toBe("load");
				expect(this).toEqual({ok: true});
				done();
			} catch(e) { done(e); }
		}.bind(this));
	});
}));

it("should bind this context on require callback (foo)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	var foo = {ok: true};
	require([], function(load) {
		try {
			expect(require("./file")).toBe("file");
			expect(this).toEqual({ok: true});
			done();
		} catch(e) { done(e); }
	}.bind(foo));
}));

it("should bind this context on require callback (foo, loaded)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	var foo = {ok: true};
	require(["./load.js"], function(load) {
		try {
			expect(require("./file")).toBe("file");
			expect(load).toBe("load");
			expect(this).toEqual({ok: true});
			done();
		} catch(e) { done(e); }
	}.bind(foo));
}));

it("should bind this context on require callback (foo)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	runWithThis({ok: true}, function() {
		require([], function(load) {
			try {
				expect(require("./file")).toBe("file");
				expect(this).toEqual({ok: {ok: true}});
				done();
			} catch(e) { done(e); }
		}.bind({ok: this}));
	});
}));

it("should bind this context on require.ensure callback", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	runWithThis({ok: true}, function() {
		require.ensure([], function(require) {
			try {
				expect(require("./file")).toBe("file");
				expect(this).toEqual({ok: true});
				done();
			} catch(e) { done(e); }
		}.bind(this));
	});
}));

it("should bind this context on require.ensure callback (loaded)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	runWithThis({ok: true}, function() {
		require.ensure(["./load.js"], function(require) {
			try {
				expect(require("./file")).toBe("file");
				expect(this).toEqual({ok: true});
				done();
			} catch(e) { done(e); }
		}.bind(this));
	});
}));
