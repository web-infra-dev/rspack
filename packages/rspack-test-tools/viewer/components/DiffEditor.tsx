import { DiffEditor as MonacoDiffEditor } from "@monaco-editor/react";
import parserBabel from "prettier/parser-babel";
import prettier from "prettier/standalone";
import type React from "react";

function formatCode(code: string): string {
	const trimedCode = code
		.split("\n")
		.filter(i => i.trim())
		.join("\n");
	return prettier.format(trimedCode, {
		parser: "babel-ts",
		plugins: [parserBabel]
	});
}

export interface IEditorProps {
	source: string;
	dist: string;
	format: boolean;
}

export const DiffEditor: React.FC<IEditorProps> = ({
	source,
	dist,
	format
}) => {
	return (
		<MonacoDiffEditor
			height="100vh"
			original={format ? formatCode(source) : source}
			modified={format ? formatCode(dist) : dist}
			language="typescript"
			options={{
				contextmenu: false,
				readOnly: true
			}}
		/>
	);
};
