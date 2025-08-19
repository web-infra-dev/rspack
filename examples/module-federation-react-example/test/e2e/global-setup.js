import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default async function globalSetup() {
	console.log("🚀 Starting Module Federation E2E Global Setup...\n");

	try {
		// First, check if dev servers are already running
		console.log("🔍 Checking if dev servers are already running...");

		let hostRunning = false;
		let remoteRunning = false;

		try {
			const response = await fetch("http://localhost:3001/");
			hostRunning = response.ok;
		} catch {}

		try {
			const response = await fetch("http://localhost:3002/remoteEntry.js");
			remoteRunning = response.ok;
		} catch {}

		if (hostRunning && remoteRunning) {
			console.log("✅ Dev servers are already running - will reuse them");

			// Set environment variable to skip Playwright webServer startup
			process.env.PLAYWRIGHT_SKIP_WEBSERVER = "true";

			// Check if we have share-usage.json files (indicating builds have run)
			const hostShareUsage = path.resolve(
				__dirname,
				"../../host/dist/share-usage.json"
			);
			const remoteShareUsage = path.resolve(
				__dirname,
				"../../remote/dist/share-usage.json"
			);

			if (fs.existsSync(hostShareUsage) && fs.existsSync(remoteShareUsage)) {
				console.log("📊 Found share-usage.json files - running optimizer...");

				try {
					execSync(
						"node --experimental-wasm-modules scripts/optimize-shared-chunks.js",
						{
							cwd: path.resolve(__dirname, "../.."),
							stdio: "inherit"
						}
					);
					console.log("✅ Optimization completed");
				} catch (error) {
					console.warn(
						"⚠️  Optimization failed, continuing with unoptimized bundles:",
						error.message
					);
				}
			} else {
				console.log(
					"⚠️  No share-usage.json found - tests will run against current bundles"
				);
			}
		} else {
			console.log(
				"🧹 Cleaning up any existing processes on ports 3001/3002..."
			);
			execSync("npx -y kill-port 3001 3002", { stdio: "ignore" });

			// Wait a moment for cleanup
			await new Promise(resolve => setTimeout(resolve, 2000));

			console.log(
				"⏳ Dev servers will be started by Playwright webServer config"
			);
		}
	} catch (error) {
		console.warn("⚠️  Setup warning:", error.message);
	}

	console.log("✅ Global setup completed\n");
}
