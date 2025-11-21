import { ConfigProvider } from "antd";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

const root = ReactDOM.createRoot(document.getElementById("root"));

root.render(
	<React.StrictMode>
		<ConfigProvider
			theme={{
				token: {
					colorPrimary: "#1890ff",
					borderRadius: 6
				}
			}}
		>
			<App />
		</ConfigProvider>
	</React.StrictMode>
);
