function normalizeUrl(url: string): string {
	const urlString = url.trim();

	if (/^data:/i.test(urlString)) {
		return urlString;
	}

	const protocol =
		urlString.indexOf("//") !== -1 ? `${urlString.split("//")[0]}//` : "";
	const components = urlString
		.replace(new RegExp(protocol, "i"), "")
		.split("/");
	const host = components[0].toLowerCase().replace(/\.$/, "");

	components[0] = "";

	const path = components
		.reduce((accumulator: string[], item) => {
			switch (item) {
				case "..":
					accumulator.pop();
					break;
				case ".":
					break;
				default:
					accumulator.push(item);
			}

			return accumulator;
		}, [])
		.join("/");

	return protocol + host + path;
}

export { normalizeUrl };
