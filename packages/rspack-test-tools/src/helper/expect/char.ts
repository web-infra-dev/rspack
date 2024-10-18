export const normalizeCLR = (str: string): string => {
	return (
		str
			.replace(/\u001b\[1m\u001b\[([0-9;]*)m/g, "<CLR=$1,BOLD>")
			.replace(/\u001b\[1m/g, "<CLR=BOLD>")
			.replace(/\u001b\[39m\u001b\[22m/g, "</CLR>")
			.replace(/\u001b\[([0-9;]*)m/g, "<CLR=$1>")
			// CHANGE: The time unit display in Rspack is second
			.replace(/[.0-9]+(<\/CLR>)?(\s?s)/g, "X$1$2")
	);
};

export const normalizeColor = (str: string): string => {
	return str.replace(/\u001b\[[0-9;]*m/g, "");
};
