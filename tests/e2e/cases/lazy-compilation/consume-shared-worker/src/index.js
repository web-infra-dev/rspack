// Create a worker that uses a shared library
const worker = new Worker(new URL("./worker.js", import.meta.url));

const root = document.getElementById("root");
root.textContent = "Main thread loaded";

// Listen for messages from worker
worker.addEventListener("message", (e) => {
	const workerResult = document.createElement("div");
	workerResult.id = "worker-result";
	workerResult.textContent = `Worker: ${e.data}`;
	root.appendChild(workerResult);
});

// Listen for errors from worker
worker.addEventListener("error", (e) => {
	const workerError = document.createElement("div");
	workerError.id = "worker-error";
	workerError.textContent = `Error: ${e.message}`;
	root.appendChild(workerError);
	console.error("Worker error:", e);
});

// Send message to worker to trigger it
worker.postMessage("start");
