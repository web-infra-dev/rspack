function getRspackVersion() {
	return require("../../package.json").version;
}

export const bindingVersionCheck = (
	bindingVersion: string,
	callback: (error: Error | null) => void
) => {
	const coreVersion = getRspackVersion();
	if (coreVersion !== bindingVersion) {
		return callback(
			new Error(
				`The rspack core version ${JSON.stringify(
					coreVersion
				)} is not match to binding version ${JSON.stringify(bindingVersion)}`
			)
		);
	} else {
		return callback(null);
	}
};
