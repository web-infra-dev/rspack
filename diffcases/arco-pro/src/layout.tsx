import useRoute from "@/routes";
import { Breadcrumb, Layout, Menu } from "@arco-design/web-react";
import {
	IconApps,
	IconCheckCircle,
	IconDashboard,
	IconExclamationCircle,
	IconFile,
	IconList,
	IconMenuFold,
	IconMenuUnfold,
	IconSettings,
	IconUser
} from "@arco-design/web-react/icon";
import cs from "classnames";
import NProgress from "nprogress";
import qs from "query-string";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { useSelector } from "react-redux";
import { Link, Redirect, Route, Switch, useHistory } from "react-router-dom";
import Footer from "./components/Footer";
import Navbar from "./components/NavBar";
import { GlobalState } from "./store";
import styles from "./style/layout.module.less";
import getUrlParams from "./utils/getUrlParams";
import { isArray } from "./utils/is";
import lazyload from "./utils/lazyload";
import useLocale from "./utils/useLocale";
const MenuItem = Menu.Item;
const SubMenu = Menu.SubMenu;

const Sider = Layout.Sider;
const Content = Layout.Content;

function getIconFromKey(key) {
	switch (key) {
		case "dashboard":
			return <IconDashboard className={styles.icon} />;
		case "list":
			return <IconList className={styles.icon} />;
		case "form":
			return <IconSettings className={styles.icon} />;
		case "profile":
			return <IconFile className={styles.icon} />;
		case "visualization":
			return <IconApps className={styles.icon} />;
		case "result":
			return <IconCheckCircle className={styles.icon} />;
		case "exception":
			return <IconExclamationCircle className={styles.icon} />;
		case "user":
			return <IconUser className={styles.icon} />;
		default:
			return <div className={styles["icon-empty"]} />;
	}
}

function getFlattenRoutes(routes) {
	const res = [];
	// function travel(_routes) {
	//   _routes.forEach((route) => {
	//     if (route.key && !route.children) {
	//       route.component = lazyload(() => import(`./pages/${route.key}`));
	//       res.push(route);
	//     } else if (isArray(route.children) && route.children.length) {
	//       travel(route.children);
	//     }
	//   });
	// }
	// travel(routes);

	// You may be very surprised why it is written this way, so am I...
	function travel(_routes) {
		_routes.forEach(route => {
			if (route.key && !route.children) {
				if (route.key.includes("dashboard/monitor")) {
					route.component = lazyload(() => import("./pages/dashboard/monitor"));
				} else if (route.key.includes("dashboard/workplace")) {
					route.component = lazyload(
						() => import("./pages/dashboard/workplace")
					);
				} else if (route.key.includes("exception/403")) {
					route.component = lazyload(() => import("./pages/exception/403"));
				} else if (route.key.includes("exception/404")) {
					route.component = lazyload(() => import("./pages/exception/404"));
				} else if (route.key.includes("exception/500")) {
					route.component = lazyload(() => import("./pages/exception/500"));
				} else if (route.key.includes("form/group")) {
					route.component = lazyload(() => import("./pages/form/group"));
				} else if (route.key.includes("form/step")) {
					route.component = lazyload(() => import("./pages/form/step"));
				} else if (route.key.includes("list/card")) {
					route.component = lazyload(() => import("./pages/list/card"));
				} else if (route.key.includes("list/search-table")) {
					route.component = lazyload(() => import("./pages/list/search-table"));
				} else if (route.key.includes("profile/basic")) {
					route.component = lazyload(() => import("./pages/profile/basic"));
				} else if (route.key.includes("result/error")) {
					route.component = lazyload(() => import("./pages/result/error"));
				} else if (route.key.includes("result/success")) {
					route.component = lazyload(() => import("./pages/result/success"));
				} else if (route.key.includes("user/info")) {
					route.component = lazyload(() => import("./pages/user/info"));
				} else if (route.key.includes("user/setting")) {
					route.component = lazyload(() => import("./pages/user/setting"));
				} else if (route.key.includes("visualization/data-analysis")) {
					route.component = lazyload(
						() => import("./pages/visualization/data-analysis")
					);
				} else if (
					route.key.includes("visualization/multi-dimension-data-analysis")
				) {
					route.component = lazyload(
						() => import("./pages/visualization/multi-dimension-data-analysis")
					);
				} else if (route.key.includes("welcome")) {
					route.component = lazyload(() => import("./pages/welcome"));
				} else if (route.key.includes("login")) {
					route.component = lazyload(() => import("./pages/login"));
				}
				res.push(route);
			} else if (isArray(route.children) && route.children.length) {
				travel(route.children);
			}
		});
	}
	travel(routes);
	return res;
}

function PageLayout() {
	const urlParams = getUrlParams();
	const history = useHistory();
	const pathname = history.location.pathname;
	const currentComponent = qs.parseUrl(pathname).url.slice(1);
	const locale = useLocale();
	const settings = useSelector((state: GlobalState) => state.settings);
	const userInfo = useSelector((state: GlobalState) => state.userInfo);

	const [routes, defaultRoute] = useRoute(userInfo?.permissions);
	const defaultSelectedKeys = [currentComponent || defaultRoute];
	const paths = (currentComponent || defaultRoute).split("/");
	const defaultOpenKeys = paths.slice(0, paths.length - 1);

	const [breadcrumb, setBreadCrumb] = useState([]);
	const [collapsed, setCollapsed] = useState<boolean>(false);
	const [selectedKeys, setSelectedKeys] =
		useState<string[]>(defaultSelectedKeys);
	const [openKeys, setOpenKeys] = useState<string[]>(defaultOpenKeys);

	const routeMap = useRef<Map<string, React.ReactNode[]>>(new Map());

	const navbarHeight = 60;
	const menuWidth = collapsed ? 48 : settings.menuWidth;

	const showNavbar = settings.navbar && urlParams.navbar !== false;
	const showMenu = settings.menu && urlParams.menu !== false;
	const showFooter = settings.footer && urlParams.footer !== false;

	const flattenRoutes = useMemo(() => getFlattenRoutes(routes) || [], [routes]);

	function renderRoutes(locale) {
		const nodes = [];
		routeMap.current.clear();
		function travel(_routes, level, parentNode = []) {
			return _routes.map(route => {
				const { breadcrumb = true } = route;

				const iconDom = getIconFromKey(route.key);
				const titleDom = (
					<>
						{iconDom} {locale[route.name] || route.name}
					</>
				);
				if (
					route.component &&
					(!isArray(route.children) ||
						(isArray(route.children) && !route.children.length))
				) {
					routeMap.current.set(
						`/${route.key}`,
						breadcrumb ? [...parentNode, route.name] : []
					);
					if (level > 1) {
						return <MenuItem key={route.key}>{titleDom}</MenuItem>;
					}
					nodes.push(
						<MenuItem key={route.key}>
							<Link to={`/${route.key}`}>{titleDom}</Link>
						</MenuItem>
					);
				}
				if (isArray(route.children) && route.children.length) {
					const parentNode = [];
					if (iconDom.props.isIcon) {
						parentNode.push(iconDom);
					}

					if (level > 1) {
						return (
							<SubMenu key={route.key} title={titleDom}>
								{travel(route.children, level + 1, [...parentNode, route.name])}
							</SubMenu>
						);
					}
					nodes.push(
						<SubMenu key={route.key} title={titleDom}>
							{travel(route.children, level + 1, [...parentNode, route.name])}
						</SubMenu>
					);
				}
			});
		}
		travel(routes, 1);
		return nodes;
	}

	function onClickMenuItem(key) {
		const currentRoute = flattenRoutes.find(r => r.key === key);
		// const component = currentRoute.component;
		// const preload = component.preload;
		const preload = new Promise(resolve => {
			setTimeout(() => {
				resolve(true);
			}, Math.random() * 500);
		});
		NProgress.start();
		preload.then(() => {
			setSelectedKeys([key]);
			history.push(currentRoute.path ? currentRoute.path : `/${key}`);
			NProgress.done();
		});
	}

	function toggleCollapse() {
		setCollapsed(collapsed => !collapsed);
	}

	const paddingLeft = showMenu ? { paddingLeft: menuWidth } : {};
	const paddingTop = showNavbar ? { paddingTop: navbarHeight } : {};
	const paddingStyle = { ...paddingLeft, ...paddingTop };

	useEffect(() => {
		const routeConfig = routeMap.current.get(pathname);
		setBreadCrumb(routeConfig || []);
	}, [pathname]);
	return (
		<Layout className={styles.layout}>
			<div
				className={cs(styles["layout-navbar"], {
					[styles["layout-navbar-hidden"]]: !showNavbar
				})}
			>
				<Navbar show={showNavbar} />
			</div>
			<Layout>
				{showMenu && (
					<Sider
						className={styles["layout-sider"]}
						width={menuWidth}
						collapsed={collapsed}
						onCollapse={setCollapsed}
						trigger={null}
						collapsible
						breakpoint="xl"
						style={paddingTop}
					>
						<div className={styles["menu-wrapper"]}>
							<Menu
								collapse={collapsed}
								onClickMenuItem={onClickMenuItem}
								selectedKeys={selectedKeys}
								openKeys={openKeys}
								onClickSubMenu={(_, openKeys) => setOpenKeys(openKeys)}
							>
								{renderRoutes(locale)}
							</Menu>
						</div>
						<div className={styles["collapse-btn"]} onClick={toggleCollapse}>
							{collapsed ? <IconMenuUnfold /> : <IconMenuFold />}
						</div>
					</Sider>
				)}
				<Layout className={styles["layout-content"]} style={paddingStyle}>
					<div className={styles["layout-content-wrapper"]}>
						{!!breadcrumb.length && (
							<div className={styles["layout-breadcrumb"]}>
								<Breadcrumb>
									{breadcrumb.map((node, index) => (
										<Breadcrumb.Item key={index}>
											{typeof node === "string" ? locale[node] || node : node}
										</Breadcrumb.Item>
									))}
								</Breadcrumb>
							</div>
						)}
						<Content>
							<Switch>
								{flattenRoutes.map((route, index) => {
									return (
										<Route
											key={index}
											path={`/${route.key}`}
											component={route.component}
										/>
									);
								})}
								<Route exact path="/">
									<Redirect to={`/${defaultRoute}`} />
								</Route>
								<Route
									path="*"
									component={lazyload(() => import("./pages/exception/403"))}
								/>
							</Switch>
						</Content>
					</div>
					{showFooter && <Footer />}
				</Layout>
			</Layout>
		</Layout>
	);
}

export default PageLayout;
