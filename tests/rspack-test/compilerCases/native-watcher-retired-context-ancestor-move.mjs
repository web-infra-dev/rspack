import assert from "node:assert/strict";
import { EventEmitter, on } from "node:events";
import fs from "node:fs";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";

function inputs(context) {
	const root = path.join(context.getDist(), "watched");
	const project = path.join(root, "project");
	const src = path.join(root, "src");
	const pages = path.join(src, "pages");
	const generated = path.join(pages, "generated");
	const nested = path.join(generated, "nested");
	return {
		root,
		project,
		src,
		pages,
		generated,
		nested,
		file: path.join(nested, "entry.js"),
		member: path.join(nested, "context-only.js"),
		marker: path.join(src, "scan-marker.js"),
		retired: path.join(root, "retired"),
		fence: path.join(project, "fence"),
		ready: path.join(root, "retired-watch", "ready.js"),
		rebound: path.join(root, "rebound-watch", "ready.js"),
	};
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [false, true].map((roundTrip) => ({
	name: roundTrip ? "restore" : "remove",
	description: roundTrip
		? "should keep watching the file after its ancestor moves out and back"
		: "should refresh retained dependencies when an ancestor moves after its context retires",
	options(context) {
		const paths = inputs(context);
		fs.rmSync(paths.root, { recursive: true, force: true });
		fs.mkdirSync(paths.project, { recursive: true });
		fs.mkdirSync(paths.nested, { recursive: true });
		fs.mkdirSync(path.dirname(paths.ready));
		fs.mkdirSync(path.dirname(paths.rebound));
		const loader = path.join(paths.project, "deps.cjs");
		const entry = path.join(paths.project, "selection.js");
		for (const file of [
			entry,
			paths.file,
			paths.member,
			paths.marker,
			paths.fence,
			paths.ready,
			paths.rebound,
		]) {
			fs.writeFileSync(file, "");
			fs.utimesSync(file, 1_600_000_000.123, 1_600_000_000.123);
		}
		fs.writeFileSync(
			loader,
			`
			const fs = require("node:fs");
			const path = require("node:path");
			const deps = ${JSON.stringify(paths)};
			module.exports = function () {
				this.cacheable(false);
				this.addDependency(deps.file);
				this.addDependency(deps.fence);
				this.addMissingDependency(deps.generated);
				this.addContextDependency(deps.nested);
				if (fs.existsSync(deps.retired)) {
					this.addContextDependency(path.dirname(deps.ready));
					if (fs.readFileSync(deps.retired, "utf8") === "rebind") {
						this.addContextDependency(path.dirname(deps.rebound));
					}
				} else {
					this.addContextDependency(deps.src);
				}
				return "module.exports = 0";
			};
		`,
		);
		return {
			mode: "development",
			devtool: false,
			context: paths.project,
			entry,
			experiments: { nativeWatcher: true },
			module: { rules: [{ test: /selection\.js$/, use: loader }] },
		};
	},
	async build(context, compiler) {
		const {
			root,
			pages,
			nested,
			file,
			member,
			marker,
			fence,
			retired,
			ready,
			rebound,
		} = inputs(context);
		const signal = AbortSignal.timeout(15_000);
		const events = new EventEmitter();
		const builds = on(events, "build", { signal });
		const watch = compiler.watch({ aggregateTimeout: 30 }, (error, stats) => {
			if (error || stats.hasErrors()) {
				events.emit("error", error ?? new Error(stats.toString()));
				return;
			}
			events.emit("build", {
				fence: compiler.modifiedFiles?.has(fence),
				edit: compiler.modifiedFiles?.has(file),
				file: compiler.fileTimestamps?.get(file),
				directory: compiler.contextTimestamps?.get(nested),
				member: compiler.fileTimestamps?.has(member),
			});
		});
		const waitFor = async (predicate) => {
			while (!predicate(compiler.watchFileSystem.getTimes())) {
				await delay(5, undefined, { signal });
			}
		};
		try {
			await builds.next();
			await waitFor((times) => marker in times && member in times);
			fs.writeFileSync(retired, "");
			watch.invalidate();
			await waitFor((times) => ready in times && !(marker in times));
			const archive = path.join(root, "archive");
			fs.renameSync(pages, archive);
			if (roundTrip) {
				fs.renameSync(archive, pages);
				fs.writeFileSync(retired, "rebind");
			}
			fs.writeFileSync(fence, "flush");
			if (roundTrip) {
				await waitFor((times) => rebound in times);
				fs.utimesSync(file, 1_600_000_010, 1_600_000_010);
			}
			for await (const [build] of builds) {
				if (roundTrip) {
					if (!build.edit || build.file?.timestamp !== 1_600_000_010_000)
						continue;
					break;
				}
				if (!build.fence) continue;
				assert.equal(build.file, null);
				assert.equal(build.directory, null);
				assert.equal(build.member, false);
				break;
			}
		} finally {
			await builds.return();
			await new Promise((resolve, reject) =>
				watch.close((error) => (error ? reject(error) : resolve())),
			);
		}
	},
}));
