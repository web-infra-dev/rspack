export default function () {
	import(/* webpackChunkName: "chunk1-a" */ "./chunk1-a");
	import(/* webpackChunkName: "chunk1-b" */ "./chunk1-b");
	import(/* webpackChunkName: "chunk1-c" */ "./chunk1-c");
}
