#!/usr/bin/env node

import { execSync, spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

console.log("🚀 Module Federation React Dev Server with Optimization\n");

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
		detached: false
	});

	const remoteProcess = spawn("pnpm", ["-C", "remote", "dev"], {
		stdio: ["ignore", "pipe", "pipe"],
		cwd: projectRoot,
		detached: false
	});

	// Forward output from dev servers
	hostProcess.stdout?.on("data", data => {
		process.stdout.write(`[HOST] ${data}`);
	});
	hostProcess.stderr?.on("data", data => {
		process.stderr.write(`[HOST] ${data}`);
	});

	remoteProcess.stdout?.on("data", data => {
		process.stdout.write(`[REMOTE] ${data}`);
	});
	remoteProcess.stderr?.on("data", data => {
		process.stderr.write(`[REMOTE] ${data}`);
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

// Build applications for optimization
function buildApplications() {
	console.log("🔨 Building applications for optimization...");
	try {
		execSync("pnpm build", {
			cwd: projectRoot,
			stdio: "inherit"
		});
		console.log("✅ Build completed");
	} catch (error) {
		throw new Error(`Build failed: ${error.message}`);
	}
}

// Run optimization
function runOptimization() {
	console.log("⚡ Running optimization on built bundles...");
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
		console.warn("⚠️  Optimization failed:", error.message);
		throw error;
	}
}

// Serve optimized build
function serveOptimizedBuild() {
	console.log("🌐 Starting optimized production servers...");

	const hostServe = spawn("pnpm", ["-C", "host", "serve"], {
		stdio: ["ignore", "pipe", "pipe"],
		cwd: projectRoot,
		detached: false
	});

	const remoteServe = spawn("pnpm", ["-C", "remote", "serve"], {
		stdio: ["ignore", "pipe", "pipe"],
		cwd: projectRoot,
		detached: false
	});

	// Forward output from serve processes
	hostServe.stdout?.on("data", data => {
		process.stdout.write(`[HOST-SERVE] ${data}`);
	});
	hostServe.stderr?.on("data", data => {
		process.stderr.write(`[HOST-SERVE] ${data}`);
	});

	remoteServe.stdout?.on("data", data => {
		process.stdout.write(`[REMOTE-SERVE] ${data}`);
	});
	remoteServe.stderr?.on("data", data => {
		process.stderr.write(`[REMOTE-SERVE] ${data}`);
	});

	return { hostServe, remoteServe };
}

// Main dev server runner
async function runDevServerWithOptimization() {
	let serverProcesses = null;
	let serveProcesses = null;

	try {
		const mode = process.argv[2] || "dev";

		if (mode === "dev") {
			// Development mode: start dev servers
			console.log("🔧 Running in development mode...");

			const serversRunning = await checkDevServers();
			if (!serversRunning) {
				serverProcesses = startDevServers();
				await waitForServers();
			} else {
				console.log("✅ Dev servers already running");
			}

			console.log("\n🌐 Development servers ready:");
			console.log("  • Host: http://localhost:3001");
			console.log("  • Remote: http://localhost:3002");
			console.log("\n📝 Press Ctrl+C to stop servers");

			// Keep process alive
			await new Promise(() => {});
		} else if (mode === "optimized" || mode === "prod") {
			// Production mode: build, optimize, and serve
			console.log("🏭 Running in production mode with optimization...");

			// Step 1: Build applications
			buildApplications();

			// Step 2: Run optimization
			runOptimization();

			// Step 3: Serve optimized build
			serveProcesses = serveOptimizedBuild();

			console.log("\n🌐 Optimized production servers ready:");
			console.log("  • Host: http://localhost:3001");
			console.log("  • Remote: http://localhost:3002");
			console.log(
				"\n📊 Bundle optimization applied - check browser network tab for reduced sizes"
			);
			console.log("📝 Press Ctrl+C to stop servers");

			// Keep process alive
			await new Promise(() => {});
		} else {
			console.error("❌ Invalid mode. Use 'dev' or 'optimized'");
			process.exit(1);
		}
	} catch (error) {
		console.error("\n❌ Dev server failed:", error.message);
		process.exit(1);
	} finally {
		// Cleanup processes on exit
		process.on("SIGINT", () => {
			console.log("\n🧹 Shutting down servers...");

			if (serverProcesses) {
				serverProcesses.hostProcess?.kill("SIGTERM");
				serverProcesses.remoteProcess?.kill("SIGTERM");
			}

			if (serveProcesses) {
				serveProcesses.hostServe?.kill("SIGTERM");
				serveProcesses.remoteServe?.kill("SIGTERM");
			}

			try {
				execSync("npx -y kill-port 3001 3002", { stdio: "ignore" });
			} catch {}

			process.exit(0);
		});
	}
}

// Handle different run modes
const mode = process.argv[2];

switch (mode) {
	case "optimized":
	case "prod":
		console.log("Running in production mode with optimization...");
		break;
	case "dev":
	default:
		console.log("Running in development mode...");
		break;
}

if (process.argv.includes("--help") || process.argv.includes("-h")) {
	console.log("\n📖 Module Federation Dev Server Usage:");
	console.log("\n  node scripts/dev-server.js [mode]");
	console.log("\n🔧 Modes:");
	console.log("  dev        Start development servers (default)");
	console.log("  optimized  Build, optimize, and serve production bundles");
	console.log("  prod       Alias for optimized");
	console.log("\n🌐 URLs:");
	console.log("  Host:   http://localhost:3001");
	console.log("  Remote: http://localhost:3002");
	console.log("\n📝 Examples:");
	console.log("  pnpm dev-server              # Development mode");
	console.log("  pnpm dev-server optimized     # Production with optimization");
	process.exit(0);
}

// Run the dev server
runDevServerWithOptimization();
