export interface WDSMetaObj {
	enforceWs?: boolean;
	version?: number;
}

export default function getWDSMetadata(SocketClient: any): WDSMetaObj {
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
