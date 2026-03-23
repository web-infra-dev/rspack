// ESM library entry that uses a Worker.
// The Worker creates a child compilation — the worker entry module
// exists in the module graph but NOT in any chunk of the parent
// compilation (orphan module). The ESM library plugin must not error.
const worker = new Worker(new URL("./worker.js", import.meta.url));

export function greet() {
	worker.postMessage("hello");
	return "hello";
}
