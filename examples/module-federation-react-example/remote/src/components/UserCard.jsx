import {
	CalendarOutlined,
	MailOutlined,
	TeamOutlined,
	UserOutlined
} from "@ant-design/icons";
import { Avatar, Card, Col, Row, Space, Tag, Typography } from "antd";
import dayjs from "dayjs";
import { capitalize } from "lodash-es";

const { Text, Title } = Typography;

const UserCard = ({ user }) => {
	const {
		name = "Unknown User",
		email = "no-email@example.com",
		avatar,
		role = "User",
		department = "General",
		joinDate = new Date().toISOString()
	} = user || {};

	const formatJoinDate = dayjs(joinDate).format("MMMM D, YYYY");
	const yearsOfService = dayjs().diff(dayjs(joinDate), "year");

	return (
		<Card style={{ width: "100%", maxWidth: 400 }}>
			<Space direction="vertical" size="middle" style={{ width: "100%" }}>
				<Row align="middle" gutter={16}>
					<Col>
						<Avatar
							size={80}
							src={avatar}
							icon={!avatar && <UserOutlined />}
							style={{ backgroundColor: "#1890ff" }}
						/>
					</Col>
					<Col>
						<Title level={4} style={{ margin: 0 }}>
							{capitalize(name)}
						</Title>
						<Tag color="blue">{role}</Tag>
					</Col>
				</Row>

				<Space direction="vertical" style={{ width: "100%" }}>
					<Space>
						<MailOutlined />
						<Text>{email}</Text>
					</Space>

					<Space>
						<TeamOutlined />
						<Text>{capitalize(department)} Department</Text>
					</Space>

					<Space>
						<CalendarOutlined />
						<Text>Joined {formatJoinDate}</Text>
					</Space>
				</Space>

				{yearsOfService > 0 && (
					<div
						style={{
							marginTop: 12,
							padding: "8px 12px",
							background: "#f0f2f5",
							borderRadius: 4
						}}
					>
						<Text type="secondary">
							{yearsOfService} {yearsOfService === 1 ? "year" : "years"} with
							the company
						</Text>
					</div>
				)}
			</Space>
		</Card>
	);
};

export default UserCard;
