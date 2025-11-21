import {
	BellOutlined,
	LogoutOutlined,
	SettingOutlined,
	UserOutlined
} from "@ant-design/icons";
import { Avatar, Badge, Dropdown, Layout, Space } from "antd";

const { Header } = Layout;

const AppHeader = () => {
	const userMenuItems = [
		{
			key: "profile",
			icon: <UserOutlined />,
			label: "Profile"
		},
		{
			key: "settings",
			icon: <SettingOutlined />,
			label: "Settings"
		},
		{
			type: "divider"
		},
		{
			key: "logout",
			icon: <LogoutOutlined />,
			label: "Logout"
		}
	];

	return (
		<Header className="app-header" style={{ padding: "0 24px" }}>
			<div style={{ flex: 1 }}>
				<h2 style={{ margin: 0 }}>Module Federation React Demo</h2>
			</div>
			<Space size="large">
				<Badge count={5} size="small">
					<BellOutlined style={{ fontSize: 18, cursor: "pointer" }} />
				</Badge>
				<Dropdown menu={{ items: userMenuItems }} placement="bottomRight">
					<Avatar
						style={{ cursor: "pointer", backgroundColor: "#1890ff" }}
						icon={<UserOutlined />}
					/>
				</Dropdown>
			</Space>
		</Header>
	);
};

export default AppHeader;
