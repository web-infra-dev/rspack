import { Alert, Space, Typography } from "antd";
import ChartWidget from "./components/ChartWidget";
import DataTable from "./components/DataTable";
import FormBuilder from "./components/FormBuilder";
import UserCard from "./components/UserCard";

const { Title } = Typography;

const App = () => {
	const handleFormSubmit = values => {
		console.log("Form submitted:", values);
	};

	return (
		<div>
			<Title level={1}>Remote Components Library</Title>

			<Alert
				message="Module Federation Remote App"
				description="This app exposes React components that can be consumed by other applications. All components below are available for remote consumption."
				type="info"
				showIcon
				style={{ marginBottom: 32 }}
			/>

			<Space direction="vertical" size="large" style={{ width: "100%" }}>
				<div className="component-demo">
					<Title level={3} className="component-title">
						UserCard Component
					</Title>
					<UserCard
						user={{
							name: "Jane Doe",
							email: "jane.doe@example.com",
							avatar: "https://api.dicebear.com/7.x/avataaars/svg?seed=Jane",
							role: "Product Manager",
							department: "Product",
							joinDate: "2021-06-15"
						}}
					/>
				</div>

				<div className="component-demo">
					<Title level={3} className="component-title">
						DataTable Component
					</Title>
					<DataTable />
				</div>

				<div className="component-demo">
					<Title level={3} className="component-title">
						ChartWidget Component
					</Title>
					<ChartWidget type="bar" />
				</div>

				<div className="component-demo">
					<Title level={3} className="component-title">
						FormBuilder Component
					</Title>
					<FormBuilder
						fields={[
							{
								type: "text",
								name: "username",
								label: "Username",
								required: true
							},
							{ type: "email", name: "email", label: "Email", required: true },
							{
								type: "password",
								name: "password",
								label: "Password",
								required: true
							},
							{
								type: "select",
								name: "role",
								label: "Role",
								options: ["Admin", "User", "Guest"],
								required: true
							}
						]}
						onSubmit={handleFormSubmit}
					/>
				</div>
			</Space>
		</div>
	);
};

export default App;
