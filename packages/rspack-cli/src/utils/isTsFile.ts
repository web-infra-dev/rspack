import path from "node:path";

const isTsFile = (configPath: string) => {
	const ext = path.extname(configPath);
	return /\.(c|m)?ts$/.test(ext);
};

export default isTsFile;
