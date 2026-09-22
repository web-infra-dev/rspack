let times = 0;
export default function loader(content) {
	times++;
	return content.replace("1", times);
};
