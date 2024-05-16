import React from "react";
import { Tree, type TreeProps } from "@arco-design/web-react";

export interface ITreeViewProps {
	treeData: TreeProps["treeData"];
	current: string;
	onChange: (selected: string) => void;
}

const TreeView: React.FC<ITreeViewProps> = React.memo(
	({ treeData, current, onChange }) => (
		<Tree
			autoExpandParent
			selectedKeys={[current]}
			treeData={treeData}
			onSelect={selected => onChange(selected[0])}
		/>
	)
);

export { TreeView };
