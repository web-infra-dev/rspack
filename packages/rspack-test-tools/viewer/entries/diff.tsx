import React from "react";
import { createRoot } from "react-dom/client";
import { DiffReportPage } from "../pages/diff-report";
import "@arco-design/web-react/dist/css/arco.css";

const root = document.getElementById("root");
if (root) {
	createRoot(root).render(
		<React.StrictMode>
			<DiffReportPage />
		</React.StrictMode>
	);
}
