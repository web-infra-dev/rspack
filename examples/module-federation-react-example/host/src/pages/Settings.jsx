import { SaveOutlined } from "@ant-design/icons";
import {
	Button,
	Card,
	Divider,
	Form,
	Input,
	Select,
	Space,
	Switch,
	Typography
} from "antd";

const { Title, Text } = Typography;

const Settings = () => {
	const [form] = Form.useForm();

	const handleSubmit = values => {
		console.log("Settings saved:", values);
		// Here you would typically save to backend/state
	};

	return (
		<div>
			<Title level={2}>Settings</Title>

			<Row gutter={[16, 16]}>
				<Col xs={24} lg={16}>
					<Card title="General Settings">
						<Form
							form={form}
							layout="vertical"
							onFinish={handleSubmit}
							initialValues={{
								appName: "Module Federation React Demo",
								language: "en",
								timezone: "UTC",
								notifications: true,
								darkMode: false
							}}
						>
							<Form.Item
								label="Application Name"
								name="appName"
								rules={[
									{ required: true, message: "Please enter application name" }
								]}
							>
								<Input />
							</Form.Item>

							<Form.Item label="Language" name="language">
								<Select>
									<Select.Option value="en">English</Select.Option>
									<Select.Option value="es">Spanish</Select.Option>
									<Select.Option value="fr">French</Select.Option>
									<Select.Option value="de">German</Select.Option>
								</Select>
							</Form.Item>

							<Form.Item label="Timezone" name="timezone">
								<Select>
									<Select.Option value="UTC">UTC</Select.Option>
									<Select.Option value="EST">Eastern Time</Select.Option>
									<Select.Option value="PST">Pacific Time</Select.Option>
									<Select.Option value="CET">
										Central European Time
									</Select.Option>
								</Select>
							</Form.Item>

							<Divider />

							<Form.Item
								label="Email Notifications"
								name="notifications"
								valuePropName="checked"
							>
								<Switch />
							</Form.Item>

							<Form.Item
								label="Dark Mode"
								name="darkMode"
								valuePropName="checked"
							>
								<Switch />
							</Form.Item>

							<Form.Item>
								<Button
									type="primary"
									htmlType="submit"
									icon={<SaveOutlined />}
								>
									Save Settings
								</Button>
							</Form.Item>
						</Form>
					</Card>
				</Col>

				<Col xs={24} lg={8}>
					<Card title="Application Info">
						<Space direction="vertical" style={{ width: "100%" }}>
							<div>
								<Text strong>Version:</Text>
								<Text> 1.0.0</Text>
							</div>
							<div>
								<Text strong>Environment:</Text>
								<Text> Development</Text>
							</div>
							<div>
								<Text strong>API Endpoint:</Text>
								<Text> http://localhost:3000/api</Text>
							</div>
							<div>
								<Text strong>Build Date:</Text>
								<Text> {new Date().toLocaleDateString()}</Text>
							</div>
						</Space>
					</Card>

					<Card title="Module Federation Info" style={{ marginTop: 16 }}>
						<Space direction="vertical" style={{ width: "100%" }}>
							<div>
								<Text strong>Host URL:</Text>
								<Text> http://localhost:3001</Text>
							</div>
							<div>
								<Text strong>Remote URL:</Text>
								<Text> http://localhost:3002</Text>
							</div>
							<div>
								<Text strong>Shared Libs:</Text>
								<Text> React, Ant Design, Redux</Text>
							</div>
						</Space>
					</Card>
				</Col>
			</Row>
		</div>
	);
};

// Import Row and Col
import { Col, Row } from "antd";

export default Settings;
