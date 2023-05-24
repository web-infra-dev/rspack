module.exports = ({ config }) => {
	// eslint-disable-next-line no-param-reassign
	config.resolve.alias = {
		react: require.resolve("react"),
		"react-dom": require.resolve("react-dom"),
		...config.resolve.alias
	};
	return config;
};
