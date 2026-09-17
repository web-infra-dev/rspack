const enableBindingTesting = !!process.env.RSPACK_BINDING;

export default function (config) {
	return enableBindingTesting
};
