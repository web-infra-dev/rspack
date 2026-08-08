module.exports = async (url) => {
	const pathname = new URL(url).pathname;

	if (pathname === "/runtime.js") {
		return {
			status: 200,
			headers: {
				"content-type": "application/javascript",
			},
			body: Buffer.from("export default {};\n"),
		};
	}

	return {
		status: 404,
		headers: {
			"content-type": "text/plain",
		},
		body: Buffer.from(`Not found: ${pathname}`),
	};
};
