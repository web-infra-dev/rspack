export default function abc() {
	if (__resourceQuery) {
		return "abc" + __resourceQuery;
	}
	return "abc";
}
