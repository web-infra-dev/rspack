function normalizeUrl(urlString: string): string {
	urlString = urlString.trim();

	if (/^data:/i.test(urlString)) {
		return urlString;
	}

	var protocol =
		urlString.indexOf("//") !== -1 ? urlString.split("//")[0] + "//" : "";
	var components = urlString.replace(new RegExp(protocol, "i"), "").split("/");
	var host = components[0].toLowerCase().replace(/\.$/, "");

	components[0] = "";

	var path = components
		.reduce(function (accumulator: string[], item) {
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
