const { spawn } = require("child_process");
const http = require("http");

console.log("Building legacy apps...");
const buildProcess = spawn("pnpm", ["legacy:build"], {
	stdio: "inherit",
	shell: true
});

buildProcess.on("close", code => {
	if (code !== 0) {
		console.error("Build failed");
		process.exit(code);
	}

	console.log("Starting servers...");
	const serveProcess = spawn("pnpm", ["serve"], {
		stdio: ["pipe", "inherit", "inherit"],
		shell: true,
		detached: true
	});

	// Wait a bit for servers to start
	setTimeout(() => {
		console.log("Checking if servers are ready...");
		checkPorts();
	}, 3000);
});

const checkPorts = () => {
	const ports = [3001, 3002, 3003, 3004, 3005];
	let readyCount = 0;

	ports.forEach(port => {
		const req = http.request(
			{
				hostname: "localhost",
				port,
				path: "/",
				method: "HEAD",
				timeout: 2000
			},
			res => {
				if (res.statusCode < 400) {
					readyCount++;
					console.log(`Port ${port} is ready (${readyCount}/${ports.length})`);
					if (readyCount === ports.length) {
						console.log("All servers are ready!");
						process.exit(0);
					}
				}
			}
		);

		req.on("error", () => {
			console.log(`Port ${port} not ready yet, retrying...`);
			setTimeout(() => checkPorts(), 1000);
		});

		req.on("timeout", () => {
			req.destroy();
			console.log(`Port ${port} timeout, retrying...`);
			setTimeout(() => checkPorts(), 1000);
		});

		req.end();
	});
};
