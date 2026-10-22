export default function (source) {
	return `module.exports = ${JSON.stringify(source + "-simple")}`;
};
