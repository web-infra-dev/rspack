import { answer } from "./data.cjs";

export { answer };

export const lazy = () => import("./data.cjs").then(m => m.default);

export const eager = async () => {
	const { default: data } = await import(
		/* webpackMode: "eager" */ "./data.cjs"
	);
	return data;
};

export const weak = () =>
	import(/* webpackMode: "weak" */ "./data.cjs").then(m => m.default);
