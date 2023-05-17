const PolyfilledBuiltinModules = {
	assert: require.resolve("assert/"),
	buffer: require.resolve("buffer/"),
	console: require.resolve("console-browserify"),
	constants: require.resolve("constants-browserify"),
	crypto: require.resolve("crypto-browserify"),
	domain: require.resolve("domain-browser"),
	events: require.resolve("events/"),
	http: require.resolve("stream-http"),
	https: require.resolve("https-browserify"),
	os: require.resolve("os-browserify/browser"),
	path: require.resolve("path-browserify"),
	punycode: require.resolve("punycode/"),
	process: require.resolve("process/browser"),
	querystring: require.resolve("querystring-es3"),
	stream: require.resolve("stream-browserify"),
	_stream_duplex: require.resolve("readable-stream/lib/_stream_duplex"),
	_stream_passthrough: require.resolve(
		"readable-stream/lib/_stream_passthrough"
	),
	_stream_readable: require.resolve("readable-stream/lib/_stream_readable"),
	_stream_transform: require.resolve("readable-stream/lib/_stream_transform"),
	_stream_writable: require.resolve("readable-stream/lib/_stream_writable"),
	string_decoder: require.resolve("string_decoder/"),
	sys: require.resolve("util/"),
	timers: require.resolve("timers-browserify"),
	tty: require.resolve("tty-browserify"),
	url: require.resolve("url/"),
	util: require.resolve("util/"),
	vm: require.resolve("vm-browserify"),
	zlib: require.resolve("browserify-zlib")
};
module.exports = class PolyfillBuiltinsPlugin {
	apply(compiler) {
		const provide = {
			Buffer: [require.resolve("buffer/"), "Buffer"],
			process: [require.resolve("process/browser")]
		};

		compiler.options.builtins = {
			...compiler.options.builtins,
			provide: {
				...compiler.options.builtins?.provide,
				...provide
			}
		};
		compiler.options.resolve.fallback = {
			...PolyfilledBuiltinModules,
			...compiler.options.resolve.fallback
		};
	}
};
