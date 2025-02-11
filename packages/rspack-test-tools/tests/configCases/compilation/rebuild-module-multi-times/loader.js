let times = 0;
module.exports = function loader(content) {
	times++;
	return content.replace("1", times);
};
