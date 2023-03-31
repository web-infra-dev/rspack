it("should allow to create a WebWorker", async () => {
	const worker = new Worker(new URL("./worker.js", import.meta.url), {
		type: "module"
	});
	worker.postMessage("ok");
	const result = await new Promise(resolve => {
		worker.onmessage = event => {
			resolve(event.data);
		};
	});
	expect(result).toBe("data: OK, thanks");
	await worker.terminate();
});

it("should allow to create a WebWorker (multiple options)", async () => {
	const worker = new Worker(new URL("./worker.js", import.meta.url), {
		type: "module",
		name: "worker1"
	});
	worker.postMessage("ok");
	const result = await new Promise(resolve => {
		worker.onmessage = event => {
			resolve(event.data);
		};
	});
	expect(result).toBe("data: OK, thanks");
	await worker.terminate();
});

it("should allow to create a WebWorker (spread type)", async () => {
	const worker = new Worker(new URL("./worker.js", import.meta.url), {
		...{ type: "module" }
	});
	worker.postMessage("ok");
	const result = await new Promise(resolve => {
		worker.onmessage = event => {
			resolve(event.data);
		};
	});
	expect(result).toBe("data: OK, thanks");
	await worker.terminate();
});

it("should allow to create a WebWorker (expression)", async () => {
	const options = { type: "module" };
	const worker = new Worker(new URL("./worker.js", import.meta.url), options);
	worker.postMessage("ok");
	const result = await new Promise(resolve => {
		worker.onmessage = event => {
			resolve(event.data);
		};
	});
	expect(result).toBe("data: OK, thanks");
	await worker.terminate();
});

it("should allow to share chunks", async () => {
	const promise = import("./module");
	const script = document.head._children[0];
	const src = script.src;
	const file = src.slice(src.lastIndexOf("/"));
	__non_webpack_require__(`./${file}`);
	script.onload();
	const { upper } = await promise;
	expect(upper("ok")).toBe("OK");
});
