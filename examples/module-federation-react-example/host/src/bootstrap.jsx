import { ConfigProvider } from "antd";
import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "react-redux";
import App from "./App";
import { store } from "./store";

const root = ReactDOM.createRoot(document.getElementById("root"));

root.render(
	<React.StrictMode>
		<Provider store={store}>
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
		</Provider>
	</React.StrictMode>
);
