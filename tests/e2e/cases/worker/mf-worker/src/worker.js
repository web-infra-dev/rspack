// Worker tries to import the shared library
import("shared-lib").then((lib) => {
	self.addEventListener("message", (e) => {
		if (e.data === "start") {
			// Use the shared library and send result back
			self.postMessage(lib.getValue());
		}
	});

	// Signal that worker is ready
	self.postMessage("Worker ready");
}).catch((err) => {
	self.postMessage(`Worker import error: ${err.message}`);
	console.error("Worker failed to import shared-lib:", err);
});
