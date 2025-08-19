import { execSync } from "node:child_process";

export default async function globalTeardown() {
	try {
		execSync("npx -y kill-port 3001 3002", { stdio: "ignore" });
	} catch {}
}
