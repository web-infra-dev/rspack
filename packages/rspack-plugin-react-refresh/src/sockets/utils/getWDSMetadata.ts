export interface WDSMetaObj {
	enforceWs?: boolean;
	version?: number;
}

declare class WebSocketClient {
	public client: WebSocket;

	constructor(url: string);

	onOpen(f: (...args: any[]) => void): void;

	onClose(f: (...args: any[]) => void): void;

	onMessage(f: (...args: any[]) => void): void;
}

export interface SocketClient {
	new (url: string): WebSocketClient;
}

export default function getWDSMetadata(SocketClient: SocketClient): WDSMetaObj {
	let enforceWs = false;
	if (
		typeof SocketClient.name !== "undefined" &&
		SocketClient.name !== null &&
		SocketClient.name.toLowerCase().includes("websocket")
	) {
		enforceWs = true;
	}

	let version;
	// WDS versions <=3.5.0
	if (!("onMessage" in SocketClient.prototype)) {
		version = 3;
	} else {
		// WDS versions >=3.5.0 <4
		if (
			"getClientPath" in SocketClient ||
			Object.getPrototypeOf(SocketClient).name === "BaseClient"
		) {
			version = 3;
		} else {
			version = 4;
		}
	}

	return {
		enforceWs: enforceWs,
		version: version
	};
}
