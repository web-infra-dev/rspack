import { sharedData } from "./shared";
import update from "@rspack/test-tools/helper/legacy/update.esm";

it("should handle HMR with runtime chunk in ESM format", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(sharedData.version).toBe("1.0.0");

	import.meta.webpackHot.accept(["./shared"]);

	NEXT(update(done, true, () => {
		import("./shared").then(updatedModule => {
			expect(updatedModule.sharedData.version).toBe("2.0.0");
			done();
		}).catch(done);
	}));
}));

it("should load async shared module with runtime chunk", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	import("./async-shared").then(module => {
		expect(module.asyncData.loaded).toBe(true);
		expect(module.asyncData.content).toBe("Async shared content");
		done();
	}).catch(done);
}));
