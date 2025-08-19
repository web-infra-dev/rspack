#!/usr/bin/env node

import { execSync, spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

console.log("🚀 Module Federation React E2E Test Runner (Dev Mode)\n");

// Check if dev servers are running
async function checkDevServers() {
	try {
		const hostResponse = await fetch("http://localhost:3001/");
		const remoteResponse = await fetch("http://localhost:3002/remoteEntry.js");
		return hostResponse.ok && remoteResponse.ok;
	} catch {
		return false;
	}
}

// Start dev servers
function startDevServers() {
	console.log("🚀 Starting dev servers...");

	const hostProcess = spawn("pnpm", ["-C", "host", "dev"], {
		stdio: ["ignore", "pipe", "pipe"],
		cwd: projectRoot,
		detached: true
	});

	const remoteProcess = spawn("pnpm", ["-C", "remote", "dev"], {
		stdio: ["ignore", "pipe", "pipe"],
		cwd: projectRoot,
		detached: true
	});

	return { hostProcess, remoteProcess };
}

// Wait for servers to be ready
async function waitForServers(timeout = 60000) {
	const start = Date.now();

	while (Date.now() - start < timeout) {
		if (await checkDevServers()) {
			console.log("✅ Dev servers are ready");
			return true;
		}
		await new Promise(resolve => setTimeout(resolve, 1000));
	}

	throw new Error("Dev servers failed to start within timeout");
}

// Run optimization
function runOptimization() {
	console.log("⚡ Running optimization...");
	try {
		execSync(
			"node --experimental-wasm-modules scripts/optimize-shared-chunks.js",
			{
				cwd: projectRoot,
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
}

// Run command and return promise
function runCommand(command, args, options = {}) {
	return new Promise((resolve, reject) => {
		console.log(`Running: ${command} ${args.join(" ")}`);
		const child = spawn(command, args, {
			stdio: "inherit",
			shell: true,
			...options
		});

		child.on("close", code => {
			if (code === 0) {
				resolve();
			} else {
				reject(new Error(`Command failed with exit code ${code}`));
			}
		});

		child.on("error", reject);
	});
}

// Main test runner
async function runE2ETests() {
	let serverProcesses = null;

	try {
		// Step 1: Check if dev servers are already running
		const serversRunning = await checkDevServers();

		if (!serversRunning) {
			console.log("🚀 Starting dev servers...");
			serverProcesses = startDevServers();
			await waitForServers();
		} else {
			console.log("✅ Dev servers already running");
		}

		// Step 2: Run optimization after servers are ready
		runOptimization();

		// Step 3: Install Playwright if needed
		console.log("Ensuring Playwright is installed...");
		try {
			await runCommand("npx", ["playwright", "install", "--with-deps"], {
				cwd: projectRoot
			});
		} catch (_error) {
			console.log("Installing Playwright...");
			await runCommand("pnpm", ["add", "-D", "@playwright/test"], {
				cwd: projectRoot
			});
			await runCommand("npx", ["playwright", "install", "--with-deps"], {
				cwd: projectRoot
			});
		}

		// Step 4: Run E2E tests
		console.log("\n🧪 Running Playwright E2E tests...");

		const testArgs = process.argv.slice(2);
		const playwrightArgs = ["playwright", "test"];

		// Add common args: default to concise list reporter unless user overrides
		if (!testArgs.some(arg => arg.startsWith("--reporter="))) {
			playwrightArgs.push("--reporter=list");
		}

		// Add user args
		playwrightArgs.push(...testArgs);

		await runCommand("npx", playwrightArgs, { cwd: projectRoot });

		console.log("\n✅ E2E tests completed successfully!");
		// HTML report serving disabled; using non-HTML reporter by default
	} catch (error) {
		console.error("\n❌ E2E tests failed:", error.message);
		process.exit(1);
	} finally {
		// Cleanup: Kill dev servers if we started them
		if (serverProcesses && !(await checkDevServers())) {
			console.log("🧹 Cleaning up dev servers...");
			try {
				if (serverProcesses.hostProcess)
					serverProcesses.hostProcess.kill("SIGTERM");
				if (serverProcesses.remoteProcess)
					serverProcesses.remoteProcess.kill("SIGTERM");
				execSync("npx -y kill-port 3001 3002", { stdio: "ignore" });
			} catch {}
		}
	}
}

// Handle different run modes
const mode = process.argv[2];

switch (mode) {
	case "ui":
		console.log("Running tests in UI mode...");
		process.argv.push("--ui");
		break;
	case "headed":
		console.log("Running tests in headed mode...");
		process.argv.push("--headed");
		break;
	case "debug":
		console.log("Running tests in debug mode...");
		process.argv.push("--debug");
		break;
	case "dev":
		console.log(
			"Running in development mode (will start/use existing servers)..."
		);
		break;
	default:
		console.log("Running tests in headless mode...");
}

// Run the tests
runE2ETests();
