export default function loader(content) {
	return `module.exports = ${JSON.stringify(content)}`;
};
