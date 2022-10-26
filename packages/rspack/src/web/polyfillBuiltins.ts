import { Compiler } from "../compiler";
const PolyfilledBuiltinModules = {
	assert: "assert/",
	buffer: "buffer/",
	console: "console-browserify",
	constants: "constants-browserify",
	crypto: "crypto-browserify",
	domain: "domain-browser",
	events: "events/",
	http: "stream-http",
	https: "https-browserify",
	os: "os-browserify/browser",
	path: "path-browserify",
	punycode: "punycode/",
	process: "process/browser",
	querystring: "querystring-es3",
	stream: "stream-browserify",
	_stream_duplex: "readable-stream/duplex",
	_stream_passthrough: "readable-stream/passthrough",
	_stream_readable: "readable-stream/readable",
	_stream_transform: "readable-stream/transform",
	_stream_writable: "readable-stream/writable",
	string_decoder: "string_decoder/",
	sys: "util/",
	timers: "timers-browserify",
	tty: "tty-browserify",
	url: "url/",
	util: "util/",
	vm: "vm-browserify",
	zlib: "browserify-zlib"
};
export class PolyfillBuiltinsPlugin {
	apply(compiler: Compiler) {
		compiler.options.resolve.alias = {
			...PolyfilledBuiltinModules,
			...compiler.options.resolve.alias
		};
	}
}
