function removeCWD(str) {
	const isWin = process.platform === "win32";
	let cwd = process.cwd();

	if (isWin) {
		// eslint-disable-next-line no-param-reassign
		str = str.replace(/\\/g, "/");
		// eslint-disable-next-line no-param-reassign
		cwd = cwd.replace(/\\/g, "/");
	}

	return str.replace(new RegExp(cwd, "g"), "");
}

export default errors =>
	errors.map(error =>
		removeCWD(error.toString().split("\n").slice(0, 2).join("\n"))
	);
