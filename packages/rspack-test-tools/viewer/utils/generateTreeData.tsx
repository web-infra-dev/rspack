import type { TreeDataType } from "@arco-design/web-react/es/Tree/interface";
import { ECompareResultType } from "../../src/type";

const DIFF_TYPE_COLOR = {
	[ECompareResultType.Same]: "#00B42A",
	[ECompareResultType.Missing]: "#86909C",
	[ECompareResultType.OnlyDist]: "#CB272D",
	[ECompareResultType.OnlySource]: "#CB272D",
	[ECompareResultType.Different]: "#F77234"
};

export const generateTreeData = (
	root: string,
	rawData: Array<{
		name: string;
		source: string;
		dist: string;
		type: ECompareResultType;
	}>
) => {
	const result: TreeDataType[] = [];
	const fileMap: Map<string, number> = new Map();
	const directoryMap: Map<string, TreeDataType> = new Map();
	for (const file of rawData) {
		let relativePath = file.name.replace(root, "");
		if (relativePath.startsWith("/")) {
			relativePath = relativePath.slice(1);
		}
		const splitPath = relativePath.split("/");
		for (let i = 0; i < splitPath.length; i++) {
			const parentPath = i === 0 ? "" : splitPath.slice(0, i).join("/");
			const parentDirectory = parentPath && directoryMap.get(parentPath);
			if (i === splitPath.length - 1) {
				const fileMeta: TreeDataType = {
					key: file.name,
					title: splitPath[i],
					data: file,
					style: {
						color: DIFF_TYPE_COLOR[file.type]
					}
				};
				if (parentDirectory) {
					if (fileMap.has(fileMeta.key!)) {
						parentDirectory.children![
							fileMap.get(fileMeta.key!)!
						].data!.wasted = true;
						parentDirectory.children![fileMap.get(fileMeta.key!)!] = fileMeta;
					} else {
						fileMap.set(fileMeta.key!, parentDirectory.children!.length);
						parentDirectory.children!.push(fileMeta);
					}
				}
			} else {
				const dirPath = splitPath.slice(0, i + 1).join("/");
				if (!directoryMap.has(dirPath)) {
					const dirMeta: TreeDataType = {
						key: dirPath,
						title: splitPath[i],
						children: []
					};
					directoryMap.set(dirPath, dirMeta);
					if (parentDirectory) {
						parentDirectory.children!.push(dirMeta);
					} else {
						result.push(dirMeta);
					}
				}
			}
		}
	}
	return result;
};
