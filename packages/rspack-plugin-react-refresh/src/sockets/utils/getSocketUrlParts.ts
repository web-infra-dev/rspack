import getCurrentScriptSource from "./getCurrentScriptSource";
import { WDSMetaObj } from "./getWDSMetadata";
export interface SocketUrlParts {
	auth?: string;
	hostname: string;
	protocol?: string;
	pathname: string;
	port?: string;
}

interface ParsedQuery {
	sockHost: string;
	sockPath: string;
	sockPort: string;
	sockProtocol?: string;
}

export default function getSocketUrlParts(
	resourceQuery?: string,
	metadata?: WDSMetaObj
): SocketUrlParts {
	if (typeof metadata === "undefined") {
		metadata = {};
	}

	/** @type {SocketUrlParts} */
	let urlParts: SocketUrlParts = {} as SocketUrlParts;

	// If the resource query is available,
	// parse it and ignore everything we received from the script host.
	if (resourceQuery) {
		const parsedQuery: ParsedQuery = {} as ParsedQuery;
		const searchParams = new URLSearchParams(resourceQuery.slice(1));
		searchParams.forEach(function (value, key) {
			// @ts-expect-error -- ignore
			parsedQuery[key] = value;
		});

		urlParts.hostname = parsedQuery.sockHost;
		urlParts.pathname = parsedQuery.sockPath;
		urlParts.port = parsedQuery.sockPort;

		// Make sure the protocol from resource query has a trailing colon
		if (parsedQuery.sockProtocol) {
			urlParts.protocol = parsedQuery.sockProtocol + ":";
		}
	} else {
		const scriptSource = getCurrentScriptSource();

		let url: URL = {} as URL;
		try {
			// The placeholder `baseURL` with `window.location.href`,
			// is to allow parsing of path-relative or protocol-relative URLs,
			// and will have no effect if `scriptSource` is a fully valid URL.
			url = new URL(scriptSource, window.location.href);
		} catch (e) {
			// URL parsing failed, do nothing.
			// We will still proceed to see if we can recover using `resourceQuery`
		}

		// Parse authentication credentials in case we need them
		if (url.username) {
			// Since HTTP basic authentication does not allow empty username,
			// we only include password if the username is not empty.
			// Result: <username> or <username>:<password>
			urlParts.auth = url.username;
			if (url.password) {
				urlParts.auth += ":" + url.password;
			}
		}

		// `file://` URLs has `'null'` origin
		if (url.origin !== "null") {
			urlParts.hostname = url.hostname;
		}

		urlParts.protocol = url.protocol;
		urlParts.port = url.port;
	}

	if (!urlParts.pathname) {
		if (metadata.version === 4) {
			// This is hard-coded in WDS v4
			urlParts.pathname = "/ws";
		} else {
			// This is hard-coded in WDS v3
			urlParts.pathname = "/sockjs-node";
		}
	}

	// Check for IPv4 and IPv6 host addresses that correspond to any/empty.
	// This is important because `hostname` can be empty for some hosts,
	// such as 'about:blank' or 'file://' URLs.
	const isEmptyHostname =
		urlParts.hostname === "0.0.0.0" ||
		urlParts.hostname === "[::]" ||
		!urlParts.hostname;
	// We only re-assign the hostname if it is empty,
	// and if we are using HTTP/HTTPS protocols.
	if (
		isEmptyHostname &&
		window.location.hostname &&
		window.location.protocol.indexOf("http") === 0
	) {
		urlParts.hostname = window.location.hostname;
	}

	// We only re-assign `protocol` when `protocol` is unavailable,
	// or if `hostname` is available and is empty,
	// since otherwise we risk creating an invalid URL.
	// We also do this when 'https' is used as it mandates the use of secure sockets.
	if (
		!urlParts.protocol ||
		(urlParts.hostname &&
			(isEmptyHostname || window.location.protocol === "https:"))
	) {
		urlParts.protocol = window.location.protocol;
	}

	// We only re-assign port when it is not available
	if (!urlParts.port) {
		urlParts.port = window.location.port;
	}

	if (!urlParts.hostname || !urlParts.pathname) {
		throw new Error(
			[
				"[React Refresh] Failed to get an URL for the socket connection.",
				"This usually means that the current executed script doesn't have a `src` attribute set.",
				"You should either specify the socket path parameters under the `devServer` key in your Rspack config, or use the `overlay` option.",
				"https://www.rspack.dev/guide/tech/react#fast-refresh"
			].join("\n")
		);
	}

	return {
		auth: urlParts.auth,
		hostname: urlParts.hostname,
		pathname: urlParts.pathname,
		protocol: urlParts.protocol,
		port: urlParts.port || undefined
	};
}
