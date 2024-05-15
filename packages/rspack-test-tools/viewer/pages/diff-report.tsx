import { TreeDataType } from "@arco-design/web-react/es/Tree/interface";
import React, { useCallback, useEffect, useState } from "react";
import { Layout } from "@arco-design/web-react";
import { ECompareResultType } from "../../src/type";
import { TreeView } from "../components/TreeView";
import { DiffEditor } from "../components/DiffEditor";
import { generateTreeData } from "../utils/generateTreeData";

const Sider = Layout.Sider;
const Content = Layout.Content;

declare var window: {
	$$diff_detail$$: TDiffStats;
};

type TDiffItem = {
	name: string;
	source: string;
	dist: string;
	type: ECompareResultType;
};

type TDiffStats = {
	root: string;
	data: Array<TDiffItem>;
};

export const DiffReportPage: React.FC<{}> = () => {
	const [stats, setStats] = useState<TDiffStats>({
		root: "",
		data: []
	});
	const [treeData, setTreeData] = useState<TreeDataType[]>([]);
	const [current, setCurrent] = useState<TDiffItem>();

	useEffect(() => {
		try {
			const stats: TDiffStats = window.$$diff_detail$$;
			if (!stats) return;
			setTreeData([
				{
					key: "summary",
					title: "Summary"
				},
				...generateTreeData(stats.root, stats.data)
			]);
			const fakeSummaryItem: TDiffItem = {
				name: "summary",
				source: stats.data
					.filter(i =>
						[
							ECompareResultType.OnlySource,
							ECompareResultType.Same,
							ECompareResultType.Different
						].includes(i.type)
					)
					.map(i => i.name)
					.sort()
					.join("\n"),
				dist: stats.data
					.filter(i =>
						[
							ECompareResultType.OnlyDist,
							ECompareResultType.Same,
							ECompareResultType.Different
						].includes(i.type)
					)
					.map(i => i.name)
					.sort()
					.join("\n"),
				type: stats.data.every(i =>
					[ECompareResultType.Different, ECompareResultType.Same].includes(
						i.type
					)
				)
					? ECompareResultType.Same
					: ECompareResultType.Different
			};
			stats.data.unshift(fakeSummaryItem);
			setStats(stats);
			setCurrent(fakeSummaryItem);
		} catch (e) {
			console.error(e);
		}
	}, []);

	const onTreeChange = useCallback(
		(name: string) => {
			const item = stats.data.find(i => i.name === name);
			if (item) {
				setCurrent(item);
			}
		},
		[stats]
	);

	return (
		<Layout hasSider>
			<Sider
				width={400}
				style={{
					overflow: "auto",
					height: "100vh",
					position: "fixed",
					left: 0,
					top: 0,
					bottom: 0,
					backgroundColor: "#fff"
				}}
			>
				{stats.data.length > 0 && current && (
					<TreeView
						onChange={onTreeChange}
						treeData={treeData}
						current={current.name}
					/>
				)}
			</Sider>
			<Layout style={{ marginLeft: 400, height: "100vh" }}>
				<Content>
					{current && (
						<DiffEditor
							format={current.name !== "summary"}
							source={current.source || ""}
							dist={current.dist || ""}
						/>
					)}
				</Content>
			</Layout>
		</Layout>
	);
};
