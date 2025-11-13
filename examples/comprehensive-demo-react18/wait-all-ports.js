const http = require("http");

const ports = [3001, 3002, 3003, 3004, 3005];
const checkPort = port => {
	return new Promise(resolve => {
		const req = http.request(
			{
				hostname: "localhost",
				port,
				path: "/",
				method: "HEAD",
				timeout: 1000
			},
			res => {
				resolve(res.statusCode < 400);
			}
		);
		req.on("error", () => resolve(false));
		req.on("timeout", () => {
			req.destroy();
			resolve(false);
		});
		req.end();
	});
};

const checkAllPorts = async () => {
	const results = await Promise.all(ports.map(checkPort));
	return results.every(Boolean);
};

const waitForAllPorts = async () => {
	console.log("Waiting for all MF servers to be ready...");
	let attempts = 0;
	const maxAttempts = 120; // 2 minutes

	while (attempts < maxAttempts) {
		if (await checkAllPorts()) {
			console.log("All MF servers are ready!");
			process.exit(0);
		}
		attempts++;
		await new Promise(resolve => setTimeout(resolve, 1000));
	}

	console.error("Timeout waiting for MF servers");
	process.exit(1);
};

waitForAllPorts();
