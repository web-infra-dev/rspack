// A module that uses a Worker — the worker creates a child compilation,
// and the worker entry module exists in the module graph but NOT in any
// chunk of the parent compilation.
const worker = new Worker(new URL("./worker.js", import.meta.url));

export function greet() {
	worker.postMessage("hello");
	return "hello";
}
