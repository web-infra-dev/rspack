import {
	ArrowDownOutlined,
	ArrowUpOutlined,
	DollarOutlined,
	UserOutlined
} from "@ant-design/icons";
import { Card, Col, Row, Space, Statistic, Typography } from "antd";
import { useEffect } from "react";
import { useDispatch, useSelector } from "react-redux";
import { fetchDashboardStats } from "../store/slices/dashboardSlice";

const { Title } = Typography;

const Dashboard = () => {
	const dispatch = useDispatch();
	const { stats, loading } = useSelector(state => state.dashboard);

	useEffect(() => {
		dispatch(fetchDashboardStats());
	}, [dispatch]);

	return (
		<div>
			<Title level={2}>Dashboard</Title>
			<Row gutter={[16, 16]} className="dashboard-stats">
				<Col xs={24} sm={12} lg={6}>
					<Card loading={loading}>
						<Statistic
							title="Total Users"
							value={stats?.totalUsers || 0}
							prefix={<UserOutlined />}
							valueStyle={{ color: "#3f8600" }}
						/>
					</Card>
				</Col>
				<Col xs={24} sm={12} lg={6}>
					<Card loading={loading}>
						<Statistic
							title="Active Users"
							value={stats?.activeUsers || 0}
							prefix={<UserOutlined />}
							suffix={
								<span style={{ fontSize: 14, color: "#3f8600" }}>
									<ArrowUpOutlined /> 8%
								</span>
							}
						/>
					</Card>
				</Col>
				<Col xs={24} sm={12} lg={6}>
					<Card loading={loading}>
						<Statistic
							title="Revenue"
							value={stats?.revenue || 0}
							prefix={<DollarOutlined />}
							precision={2}
							valueStyle={{ color: "#1890ff" }}
						/>
					</Card>
				</Col>
				<Col xs={24} sm={12} lg={6}>
					<Card loading={loading}>
						<Statistic
							title="Growth"
							value={stats?.growth || 0}
							suffix="%"
							valueStyle={{ color: stats?.growth > 0 ? "#3f8600" : "#cf1322" }}
							prefix={
								stats?.growth > 0 ? <ArrowUpOutlined /> : <ArrowDownOutlined />
							}
						/>
					</Card>
				</Col>
			</Row>

			<Row gutter={[16, 16]}>
				<Col xs={24} lg={16}>
					<Card title="Recent Activity" loading={loading}>
						<Space direction="vertical" style={{ width: "100%" }}>
							<div>User John Doe completed a purchase - $125.00</div>
							<div>New user registration: jane.smith@example.com</div>
							<div>System maintenance scheduled for next week</div>
							<div>Revenue target achieved for Q4</div>
						</Space>
					</Card>
				</Col>
				<Col xs={24} lg={8}>
					<Card title="Quick Stats" loading={loading}>
						<Space direction="vertical" style={{ width: "100%" }}>
							<div>Conversion Rate: 3.2%</div>
							<div>Avg. Session Duration: 5m 32s</div>
							<div>Bounce Rate: 42%</div>
							<div>Page Views: 125,432</div>
						</Space>
					</Card>
				</Col>
			</Row>
		</div>
	);
};

export default Dashboard;
