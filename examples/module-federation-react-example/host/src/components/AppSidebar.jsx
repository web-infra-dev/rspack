import {
	AppstoreOutlined,
	BarChartOutlined,
	DashboardOutlined,
	SettingOutlined,
	UserOutlined
} from "@ant-design/icons";
import { Layout, Menu } from "antd";
import { useLocation, useNavigate } from "react-router-dom";

const { Sider } = Layout;

const AppSidebar = () => {
	const navigate = useNavigate();
	const location = useLocation();

	const menuItems = [
		{
			key: "/dashboard",
			icon: <DashboardOutlined />,
			label: "Dashboard"
		},
		{
			key: "/analytics",
			icon: <BarChartOutlined />,
			label: "Analytics"
		},
		{
			key: "/users",
			icon: <UserOutlined />,
			label: "Users"
		},
		{
			key: "/remote-components",
			icon: <AppstoreOutlined />,
			label: "Remote Components"
		},
		{
			key: "/settings",
			icon: <SettingOutlined />,
			label: "Settings"
		}
	];

	const handleMenuClick = ({ key }) => {
		navigate(key);
	};

	return (
		<Sider width={250} theme="dark">
			<div
				style={{
					height: 64,
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					borderBottom: "1px solid rgba(255, 255, 255, 0.1)"
				}}
			>
				<h3 style={{ color: "#fff", margin: 0 }}>MF React App</h3>
			</div>
			<Menu
				theme="dark"
				mode="inline"
				selectedKeys={[location.pathname]}
				items={menuItems}
				onClick={handleMenuClick}
			/>
		</Sider>
	);
};

export default AppSidebar;
