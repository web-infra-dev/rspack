export default function () {
	return import(/* webpackChunkName: "lazy" */ "./lazy");
}
---
export default function () {
	return Promise.resolve();
}
