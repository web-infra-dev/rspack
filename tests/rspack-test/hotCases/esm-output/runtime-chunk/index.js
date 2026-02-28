import { sharedData } from "./shared";

it("should handle HMR with runtime chunk in ESM format", async () => {
	expect(sharedData.version).toBe("1.0.0");
	await NEXT_HMR();
	const updatedModule = await import("./shared");
	expect(updatedModule.sharedData.version).toBe("2.0.0");
});

it("should load async shared module with runtime chunk", async () => {
	const m = await import("./async-shared");
	expect(m.asyncData.loaded).toBe(true);
	expect(m.asyncData.content).toBe("Async shared content");
});

import.meta.webpackHot.accept(["./shared", "./async-shared"]);