import assert from "assert";
import inspector from "inspector";
import fs from "fs";

export type JavascriptCPUProfile = {
	kind: "js cpu profile";
	output: string;
};
export type RustTrace = {
	kind: "rust trace";
	filter: string;
	output: string;
};

export type ProfileOptions = JavascriptCPUProfile | RustTrace;

const defaultRustTraceOutput = `./rspack.trace`;
const defaultJsCPUProfileOutput = `./rspack.jscpuprofile`;
const defaultRustTraceFilter = "trace";

export function resolveProfile(value: string): ProfileOptions[] {
	if (value.toUpperCase() === "ALL") {
		return [
			{
				kind: "rust trace",
				filter: defaultRustTraceFilter,
				output: defaultRustTraceOutput
			},
			{ kind: "js cpu profile", output: defaultJsCPUProfileOutput }
		];
	}
	return value
		.split("|")
		.map(resolveProfileOptions)
		.filter((p): p is ProfileOptions => p !== undefined);
}

function resolveProfileOptions(value: string): ProfileOptions | undefined {
	if (value.toUpperCase().startsWith("TRACE")) {
		return resolveRustTrace(value);
	}
	if (value.toUpperCase().startsWith("JSCPU")) {
		return { kind: "js cpu profile", output: defaultJsCPUProfileOutput };
	}
}

// TRACE=trace
function resolveRustTrace(value: string): RustTrace {
	const [kind, filter] = [value.slice(0, 5), value.slice(6)];
	assert(
		kind.toUpperCase() === "TRACE",
		"value should start with TRACE in resolveRustTrace"
	);
	return {
		kind: "rust trace",
		filter: filter || defaultRustTraceFilter,
		output: defaultRustTraceOutput
	};
}

export class RspackJsCPUProfilePlugin {
	constructor(private output: string) {}

	apply(compiler) {
		const session = new inspector.Session();
		session.connect();
		session.post("Profiler.enable");
		session.post("Profiler.start");
		compiler.hooks.done.tapAsync(
			RspackJsCPUProfilePlugin.name,
			(stats, callback) => {
				if (compiler.watchMode) return callback();
				session.post("Profiler.stop", (error, param) => {
					if (error) {
						console.error("Failed to generate JS CPU profile:", error);
						return;
					}
					fs.writeFileSync(this.output, JSON.stringify(param.profile));
				});
			}
		);
	}
}
