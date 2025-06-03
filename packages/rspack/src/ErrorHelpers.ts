const loaderFlag = "LOADER_EXECUTION";

const cutOffByFlag = (stack: string, flag: string) => {
	const stacks = stack.split("\n");
	for (let i = 0; i < stacks.length; i++) {
		if (stacks[i].includes(flag)) {
			stacks.length = i;
		}
	}
	return stacks.join("\n");
};

export const cutOffLoaderExecution = (stack: string) =>
	cutOffByFlag(stack, loaderFlag);
