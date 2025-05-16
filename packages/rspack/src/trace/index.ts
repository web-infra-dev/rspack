import fs from "node:fs";

// following chrome trace event format https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview?tab=t.0#heading=h.uxpopqvbjezh
export interface ChromeEvent {
	name: string;
	ph?: string;
	cat?: string; // cat is used to show different track in perfetto with id
	ts?: number;
	pid?: number;
	tid?: number;
	id?: number; // updated to allow string id
	args?: {
		[key: string]: any;
	};
	id2?: {
		local?: string;
		global?: string;
	};
}

// this is a tracer for nodejs
// FIXME: currently we only support chrome layer and do nothing for logger layer
export class JavaScriptTracer {
	// baseline time, we use offset time for tracing to align with rust side time
	static startTime: number;
	static events: ChromeEvent[];
	static layer: string;
	// tracing file path, same as rust tracing-chrome's
	static output: string;
	// inspector session for CPU Profiler
	static session: import("node:inspector").Session;

	static async initJavaScriptTrace(layer: string, output: string) {
		const { Session } = await import("node:inspector");
		this.session = new Session();
		this.layer = layer;
		this.output = output;
		this.events = [];
		const hrtime = process.hrtime();
		this.startTime = hrtime[0] * 1000000 + Math.round(hrtime[1] / 1000); // use microseconds
	}

	static initCpuProfiler() {
		if (this.layer === "chrome") {
			this.session.connect();
			this.session.post("Profiler.enable");
			this.session.post("Profiler.start");
		}
	}

	static async cleanupJavaScriptTrace() {
		if (!this.layer.includes("chrome")) {
			return;
		}

		this.session.post("Profiler.stop", (err, param) => {
			let cpu_profile: import("node:inspector").Profiler.Profile | undefined;
			if (err) {
				console.error("Error stopping profiler:", err);
			} else {
				cpu_profile = param.profile;
			}
			if (cpu_profile) {
				// add event contains cpu_profile to show cpu profile in trace viewer (firefox profiler and perfetto)
				// more info in https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview?tab=t.0#heading=h.yr4qxyxotyw
				this.pushEvent({
					name: "Profile",
					ph: "P",
					id: 1,
					...this.getCommonEv(),
					cat: "disabled-by-default-v8.cpu_profiler",
					pid: 3, // separate process id for cpu profile
					args: {
						data: {
							startTime: 0 // use offset time to align with other trace data
						}
					}
				});
				this.pushEvent({
					name: "ProfileChunk",
					ph: "P",
					id: 1,
					...this.getCommonEv(),
					cat: "disabled-by-default-v8.cpu_profiler",
					pid: 3,
					args: {
						data: {
							cpuProfile: cpu_profile,
							timeDeltas: cpu_profile.timeDeltas
						}
					}
				});
			}
			const originTrace = fs.readFileSync(this.output, "utf-8");
			// this is hack, [] is empty and [{}] is not empty
			const originTraceIsEmpty = !originTrace.includes("{");
			const eventMsg =
				(this.events.length > 0 && !originTraceIsEmpty ? "," : "") +
				this.events
					.map(x => {
						return JSON.stringify(x);
					})
					.join(",\n");

			// a naive implementation to merge rust & Node.js trace, we can't use JSON.parse because sometime the trace file is too big to parse
			const newTrace = originTrace.replace(/]$/, `${eventMsg}\n]`);
			fs.writeFileSync(this.output, newTrace, {
				flush: true
			});
		});
	}
	// get elapsed time since start(microseconds same as rust side timestamp)
	static getTs() {
		const hrtime = process.hrtime();
		return hrtime[0] * 1000000 + Math.round(hrtime[1] / 1000) - this.startTime;
	}
	// get common chrome event
	static getCommonEv() {
		return {
			tid: 1,
			pid: 2, // fake pid for detailed track
			ts: this.getTs(),
			cat: "rspack"
		};
	}
	static pushEvent(event: ChromeEvent) {
		// set id2 as perfetto track id
		if (!event.id2 && event.args?.id2) {
			event.id2 = {
				local: event.args.id2
			};
		}

		this.events.push(event);
	}
	// start an chrome async event
	static startAsync(events: ChromeEvent) {
		if (this.layer !== "chrome") {
			return;
		}
		this.pushEvent({
			...this.getCommonEv(),
			ph: "b",

			...events
		});
	}
	// end an chrome async event
	static endAsync(events: ChromeEvent) {
		if (this.layer !== "chrome") {
			return;
		}
		this.pushEvent({
			...this.getCommonEv(),
			ph: "e",
			...events
		});
	}
}
