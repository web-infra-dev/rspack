it("should require existing module with supplied error callback", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.ensure(['./file'], function(){
		try {
			var file = require('./file');
			expect(file).toBe("file");
			done();
		} catch(e) { done(e); }
	}, function(error) {});
}));

it("should call error callback on missing module", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.ensure(['./missingModule'], function(){
		try {
			require('./missingModule');
		} catch(e) { done(e); }
	}, function(error) {
		expect(error).toBeInstanceOf(Error);
		expect(error.message).toBe("Cannot find module './missingModule'");
		done();
	});
}));

it("should call error callback on missing module in context", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	(function(module) {
		require.ensure([], function(){
			require('./' + module);
		}, function(error) {
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("Cannot find module './missingModule'");
			done();
		});
	})('missingModule');
}));

it("should call error callback on exception thrown in loading module", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.ensure(['./throwing'], function(){
		try {
			require('./throwing');
		} catch(e) { done(e); }
	}, function(error) {
		expect(error).toBeInstanceOf(Error);
		expect(error.message).toBe('message');
		done();
	});
}));

it("should not call error callback on exception thrown in require callback", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.ensure(['./throwing'], function() {
		try {
			throw new Error('message');
		} catch(e) { done(e); }
	}, function(error) {
		expect(error).toBeInstanceOf(Error);
		expect(error.message).toBe('message');
		done();
	});
}));

it("should call error callback when there is an error loading the chunk", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	var temp = __webpack_chunk_load__;
	__webpack_chunk_load__ = function() { return Promise.resolve().then(function() { throw 'fake chunk load error'; }); };
	require.ensure(['./file'], function(){
		try {
			var file = require('./file');
		} catch(e) { done(e); }
	}, function(error) {
		expect(error).toBe('fake chunk load error');
		done();
	});
	__webpack_chunk_load__ = temp;
}));
