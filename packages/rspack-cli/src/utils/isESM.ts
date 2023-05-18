const isESM = async (modulePath, cwd?: string) => {
	const { readPackageUp } = await import("read-pkg-up");
	const { packageJson } = await readPackageUp({ cwd });
	if (packageJson.type === "module") {
		return true;
	}
	if (/\.(mjs|mts)$/.test(modulePath)) {
		return true;
	}
	return false;
};

export default isESM;
