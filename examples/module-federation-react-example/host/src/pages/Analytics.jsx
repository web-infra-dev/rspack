import { Card, Col, Row, Select, Space, Spin, Typography } from "antd";
import {
	ArcElement,
	CategoryScale,
	Chart as ChartJS,
	Title as ChartTitle,
	Legend,
	LinearScale,
	LineElement,
	PointElement,
	Tooltip
} from "chart.js";
import { useEffect } from "react";
import { Doughnut, Line } from "react-chartjs-2";
import { useDispatch, useSelector } from "react-redux";
import {
	fetchAnalyticsData,
	setTimeRange
} from "../store/slices/analyticsSlice";

ChartJS.register(
	CategoryScale,
	LinearScale,
	PointElement,
	LineElement,
	ChartTitle,
	Tooltip,
	Legend,
	ArcElement
);

const { Title } = Typography;

const Analytics = () => {
	const dispatch = useDispatch();
	const { data, loading, timeRange } = useSelector(state => state.analytics);

	useEffect(() => {
		dispatch(fetchAnalyticsData());
	}, [dispatch]);

	const handleTimeRangeChange = value => {
		dispatch(setTimeRange(value));
	};

	const chartOptions = {
		responsive: true,
		maintainAspectRatio: false,
		plugins: {
			legend: {
				position: "top"
			}
		}
	};

	return (
		<div>
			<Row justify="space-between" align="middle" style={{ marginBottom: 24 }}>
				<Col>
					<Title level={2}>Analytics</Title>
				</Col>
				<Col>
					<Select
						value={timeRange}
						onChange={handleTimeRangeChange}
						style={{ width: 200 }}
						options={[
							{ value: "7days", label: "Last 7 days" },
							{ value: "30days", label: "Last 30 days" },
							{ value: "6months", label: "Last 6 months" },
							{ value: "1year", label: "Last year" }
						]}
					/>
				</Col>
			</Row>

			{loading ? (
				<div className="loading-container">
					<Spin size="large" />
				</div>
			) : (
				<Row gutter={[16, 16]}>
					<Col xs={24} lg={12}>
						<Card title="Revenue Trend">
							<div style={{ height: 300 }}>
								{data?.revenue && (
									<Line data={data.revenue} options={chartOptions} />
								)}
							</div>
						</Card>
					</Col>
					<Col xs={24} lg={12}>
						<Card title="User Growth">
							<div style={{ height: 300 }}>
								{data?.userGrowth && (
									<Line data={data.userGrowth} options={chartOptions} />
								)}
							</div>
						</Card>
					</Col>
					<Col xs={24} lg={8}>
						<Card title="Device Categories">
							<div style={{ height: 300 }}>
								{data?.categories && (
									<Doughnut data={data.categories} options={chartOptions} />
								)}
							</div>
						</Card>
					</Col>
					<Col xs={24} lg={16}>
						<Card title="Key Metrics">
							<Space
								direction="vertical"
								size="large"
								style={{ width: "100%" }}
							>
								<Row gutter={16}>
									<Col span={8}>
										<Statistic title="Page Views" value="1,234,567" />
									</Col>
									<Col span={8}>
										<Statistic title="Unique Visitors" value="456,789" />
									</Col>
									<Col span={8}>
										<Statistic title="Avg. Session" value="5m 32s" />
									</Col>
								</Row>
							</Space>
						</Card>
					</Col>
				</Row>
			)}
		</div>
	);
};

// Import Statistic component
import { Statistic } from "antd";

export default Analytics;
