import "./style/global.less";
import { ConfigProvider } from "@arco-design/web-react";
import enUS from "@arco-design/web-react/es/locale/en-US";
import zhCN from "@arco-design/web-react/es/locale/zh-CN";
import axios from "axios";
import React, { Suspense, useEffect } from "react";
import ReactDOM from "react-dom";
import { Provider } from "react-redux";
import { BrowserRouter, Route, Switch } from "react-router-dom";
import { createStore } from "redux";
import { GlobalContext } from "./context";
import PageLayout from "./layout";
import "./mock";
import Login from "./pages/login";
import rootReducer from "./store";

import changeTheme from "./utils/changeTheme";
import checkLogin from "./utils/checkLogin";
import useStorage from "./utils/useStorage";

const store = createStore(rootReducer);

function Index() {
	const [lang, setLang] = useStorage("arco-lang", "en-US");
	const [theme, setTheme] = useStorage("arco-theme", "light");

	function getArcoLocale() {
		switch (lang) {
			case "zh-CN":
				return zhCN;
			case "en-US":
				return enUS;
			default:
				return zhCN;
		}
	}

	function fetchUserInfo() {
		axios.get("/api/user/userInfo").then(res => {
			store.dispatch({
				type: "update-userInfo",
				payload: { userInfo: res.data }
			});
		});
	}

	useEffect(() => {
		// if (checkLogin()) {
		//   fetchUserInfo();
		// } else if (window.location.pathname.replace(/\//g, '') !== 'login') {
		//   window.location.pathname = '/login';
		// }
	}, []);

	useEffect(() => {
		changeTheme(theme);
	}, [theme]);

	const contextValue = {
		lang,
		setLang,
		theme,
		setTheme
	};

	return (
		<BrowserRouter>
			<Suspense fallback={<div>loading....</div>}>
				<ConfigProvider
					locale={getArcoLocale()}
					componentConfig={{
						Card: {
							bordered: false
						},
						List: {
							bordered: false
						},
						Table: {
							border: false
						}
					}}
				>
					<Provider store={store}>
						<GlobalContext.Provider value={contextValue}>
							<Switch>
								<Route path="/login" component={Login} />
								<Route path="/" component={PageLayout} />
							</Switch>
						</GlobalContext.Provider>
					</Provider>
				</ConfigProvider>
			</Suspense>
		</BrowserRouter>
	);
}

ReactDOM.render(<Index />, document.getElementById("root"));
