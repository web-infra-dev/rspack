export default (config) => {
	const [major] = process.versions.node.split(".").map(Number);
	// TODO: oom
	return config.target === "web" && major >= 18;
};
