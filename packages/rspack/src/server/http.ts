import http from "http";
import type * as Connect from "connect";

export function createHttpServer(app: Connect.Server): http.Server {
	return http.createServer(app);
}

export function closeHttpServer(server?: http.Server) {
	if (!server) {
		return;
	}

	server.close(error => {
		console.log("http closed");
	});
}
