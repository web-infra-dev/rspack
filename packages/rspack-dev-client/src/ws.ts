type Fn = (...args: any[]) => void;

export function createWebSocketClient(url: string) {
	const client = new WebSocket(url, "web-server");
	client.onerror = error => {
		console.error(error);
	};
	return {
		onOpen(f: Fn) {
			client.onopen = f;
		},
		onClose(f: Fn) {
			client.onclose = f;
		},
		onMessage(f: Fn) {
			client.onmessage = e => {
				f(e.data);
			};
		}
	};
}
