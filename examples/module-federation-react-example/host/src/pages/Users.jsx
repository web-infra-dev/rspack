import { DeleteOutlined, EditOutlined } from "@ant-design/icons";
import { Button, Select, Space, Table, Tag, Typography } from "antd";
import { useEffect } from "react";
import { useDispatch, useSelector } from "react-redux";
import { fetchUsers, sortUsers } from "../store/slices/usersSlice";

const { Title } = Typography;

const Users = () => {
	const dispatch = useDispatch();
	const { list, loading, sortField } = useSelector(state => state.users);

	useEffect(() => {
		dispatch(fetchUsers());
	}, [dispatch]);

	const handleSortChange = value => {
		dispatch(sortUsers(value));
	};

	const columns = [
		{
			title: "Name",
			dataIndex: "name",
			key: "name"
		},
		{
			title: "Email",
			dataIndex: "email",
			key: "email"
		},
		{
			title: "Role",
			dataIndex: "role",
			key: "role",
			render: role => {
				const color =
					role === "Admin" ? "red" : role === "Manager" ? "blue" : "green";
				return <Tag color={color}>{role}</Tag>;
			}
		},
		{
			title: "Status",
			dataIndex: "status",
			key: "status",
			render: status => (
				<Tag color={status === "active" ? "green" : "default"}>
					{status.toUpperCase()}
				</Tag>
			)
		},
		{
			title: "Actions",
			key: "actions",
			render: (_, record) => (
				<Space size="middle">
					<Button
						type="link"
						icon={<EditOutlined />}
						onClick={() => console.log("Edit", record)}
					>
						Edit
					</Button>
					<Button
						type="link"
						danger
						icon={<DeleteOutlined />}
						onClick={() => console.log("Delete", record)}
					>
						Delete
					</Button>
				</Space>
			)
		}
	];

	return (
		<div>
			<Row justify="space-between" align="middle" style={{ marginBottom: 24 }}>
				<Col>
					<Title level={2}>Users</Title>
				</Col>
				<Col>
					<Space>
						<span>Sort by:</span>
						<Select
							value={sortField}
							onChange={handleSortChange}
							style={{ width: 150 }}
							options={[
								{ value: "name", label: "Name" },
								{ value: "email", label: "Email" },
								{ value: "role", label: "Role" }
							]}
						/>
						<Button type="primary">Add User</Button>
					</Space>
				</Col>
			</Row>

			<Table
				columns={columns}
				dataSource={list}
				loading={loading}
				rowKey="id"
				pagination={{
					pageSize: 10,
					showTotal: total => `Total ${total} users`
				}}
			/>
		</div>
	);
};

// Import Row and Col
import { Col, Row } from "antd";

export default Users;
