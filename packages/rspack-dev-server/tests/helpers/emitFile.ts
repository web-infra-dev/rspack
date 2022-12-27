import fs from "node:fs";
import WebSocket from "ws";

async function waiting(ms: number): Promise<void> {
	return new Promise(resolve => {
		setTimeout(() => {
			resolve(undefined);
		}, ms);
	});
}

export async function editFile(
	filename: string,
	replacer: (str: string) => string
): Promise<void> {
	const content = fs.readFileSync(filename, "utf-8");
	const modified = replacer(content);
	fs.writeFileSync(filename, modified);
	return waiting(1000);
}

export async function waitingForBuild(port: number | string) {
	await new Promise(resolve => {
		const ws = new WebSocket(`ws://127.0.0.1:${port}/ws`, {
			headers: {
				host: `127.0.0.1:${port}`,
				origin: `http://127.0.0.1:${port}`
			}
		});

		let opened = false;
		let received = false;
		let errored = false;

		ws.on("error", error => {
			// @ts-ignore
			if (/404/.test(error)) {
				errored = true;
			} else {
				errored = true;
			}

			ws.close();
		});

		ws.on("open", () => {
			opened = true;
		});

		ws.on("message", data => {
			// @ts-ignore
			const message = JSON.parse(data);

			if (message.type === "ok") {
				received = true;

				ws.close();
			}
		});

		ws.on("close", () => {
			if (opened && received && !errored) {
				resolve(undefined);
			} else if (errored) {
				resolve(undefined);
			}
		});
	});
}
