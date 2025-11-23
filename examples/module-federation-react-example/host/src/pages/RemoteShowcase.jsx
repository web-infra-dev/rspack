import {
	AppstoreOutlined,
	BarChartOutlined,
	FormOutlined,
	TableOutlined
} from "@ant-design/icons";
import { Alert, Card, Spin, Tabs, Typography } from "antd";
import { lazy, Suspense, useState } from "react";

const { Title } = Typography;

// Lazy load remote components
const UserCard = lazy(() => import("remote/UserCard"));
const DataTable = lazy(() => import("remote/DataTable"));
const ChartWidget = lazy(() => import("remote/ChartWidget"));
const FormBuilder = lazy(() => import("remote/FormBuilder"));

const RemoteComponentWrapper = ({ children, title }) => (
	<Card title={title} className="remote-component-wrapper">
		<Suspense
			fallback={
				<div style={{ textAlign: "center", padding: 40 }}>
					<Spin size="large" />
					<p>Loading remote component...</p>
				</div>
			}
		>
			{children}
		</Suspense>
	</Card>
);

const RemoteShowcase = () => {
	const [activeTab, setActiveTab] = useState("1");

	const tabItems = [
		{
			key: "1",
			label: (
				<span>
					<AppstoreOutlined />
					User Card
				</span>
			),
			children: (
				<RemoteComponentWrapper title="User Profile Card Component">
					<UserCard
						user={{
							name: "John Doe",
							email: "john.doe@example.com",
							avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=John",
							role: "Senior Developer",
							department: "Engineering",
							joinDate: "2022-01-15"
						}}
					/>
				</RemoteComponentWrapper>
			)
		},
		{
			key: "2",
			label: (
				<span>
					<TableOutlined />
					Data Table
				</span>
			),
			children: (
				<RemoteComponentWrapper title="Advanced Data Table Component">
					<DataTable />
				</RemoteComponentWrapper>
			)
		},
		{
			key: "3",
			label: (
				<span>
					<BarChartOutlined />
					Charts
				</span>
			),
			children: (
				<RemoteComponentWrapper title="Chart Widgets">
					<ChartWidget type="line" />
				</RemoteComponentWrapper>
			)
		},
		{
			key: "4",
			label: (
				<span>
					<FormOutlined />
					Form Builder
				</span>
			),
			children: (
				<RemoteComponentWrapper title="Dynamic Form Builder">
					<FormBuilder
						fields={[
							{
								type: "text",
								name: "firstName",
								label: "First Name",
								required: true
							},
							{
								type: "text",
								name: "lastName",
								label: "Last Name",
								required: true
							},
							{ type: "email", name: "email", label: "Email", required: true },
							{
								type: "select",
								name: "department",
								label: "Department",
								options: ["Engineering", "Sales", "Marketing", "HR"]
							}
						]}
						onSubmit={values => console.log("Form submitted:", values)}
					/>
				</RemoteComponentWrapper>
			)
		}
	];

	return (
		<div>
			<Title level={2}>Remote Components Showcase</Title>

			<Alert
				message="Module Federation in Action"
				description="These components are loaded dynamically from a remote application running on port 3002. They share React, Ant Design, and other dependencies with the host application."
				type="info"
				showIcon
				style={{ marginBottom: 24 }}
			/>

			<Tabs activeKey={activeTab} onChange={setActiveTab} items={tabItems} />
		</div>
	);
};

export default RemoteShowcase;
