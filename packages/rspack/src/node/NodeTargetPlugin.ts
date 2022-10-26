import { Compiler } from "../compiler";

const builtins = [
	"assert",
	"async_hooks",
	"buffer",
	"child_process",
	"cluster",
	"console",
	"constants",
	"crypto",
	"dgram",
	"diagnostics_channel",
	"dns",
	"dns/promises",
	"domain",
	"events",
	"fs",
	"fs/promises",
	"http",
	"http2",
	"https",
	"inspector",
	"module",
	"net",
	"os",
	"path",
	"path/posix",
	"path/win32",
	"perf_hooks",
	"process",
	"punycode",
	"querystring",
	"readline",
	"repl",
	"stream",
	"stream/promises",
	"stream/web",
	"string_decoder",
	"sys",
	"timers",
	"timers/promises",
	"tls",
	"trace_events",
	"tty",
	"url",
	"util",
	"util/types",
	"v8",
	"vm",
	"wasi",
	"worker_threads",
	"zlib",
	/^node:/,

	// cspell:word pnpapi
	// Yarn PnP adds pnpapi as "builtin"
	"pnpapi"
];
// We do not support externalsType.node-commonjs yet, so I have to use eval to hack around the limitation
function toEval(modName) {
	return `eval('require("${modName}")')`;
}
export class NodeTargetPlugin {
	apply(compiler: Compiler) {
		const externals = Object.fromEntries(builtins.map(x => [x, toEval(x)]));
		compiler.options.externals = {
			...externals,
			...compiler.options.externals
		};
	}
}
