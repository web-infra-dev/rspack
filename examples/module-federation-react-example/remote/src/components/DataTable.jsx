import { SearchOutlined } from "@ant-design/icons";
import { Button, Input, Space, Table, Tag } from "antd";
import dayjs from "dayjs";
import { debounce } from "lodash-es";
import { useCallback, useEffect, useMemo, useState } from "react";

// Default sample data if none provided - moved outside component to prevent re-creation
const DEFAULT_DATA = [
	{
		key: "1",
		product: "iPhone 14 Pro",
		category: "Electronics",
		price: 999,
		stock: 45,
		status: "available",
		lastUpdated: "2024-01-15"
	},
	{
		key: "2",
		product: "MacBook Pro M3",
		category: "Computers",
		price: 2499,
		stock: 12,
		status: "low-stock",
		lastUpdated: "2024-01-14"
	},
	{
		key: "3",
		product: "AirPods Pro",
		category: "Accessories",
		price: 249,
		stock: 0,
		status: "out-of-stock",
		lastUpdated: "2024-01-13"
	},
	{
		key: "4",
		product: "iPad Air",
		category: "Tablets",
		price: 599,
		stock: 67,
		status: "available",
		lastUpdated: "2024-01-12"
	}
];

const DataTable = ({ data: propData, columns: propColumns }) => {
	const [filteredData, setFilteredData] = useState([]);
	const [_searchText, setSearchText] = useState("");
	const [_searchedColumn, setSearchedColumn] = useState("");

	const data = useMemo(() => propData || DEFAULT_DATA, [propData]);

	useEffect(() => {
		setFilteredData(data);
	}, [data]);

	const handleSearch = useCallback(
		debounce((selectedKeys, confirm, dataIndex) => {
			confirm();
			setSearchText(selectedKeys[0]);
			setSearchedColumn(dataIndex);
		}, 300),
		[]
	);

	const handleReset = useCallback(clearFilters => {
		clearFilters();
		setSearchText("");
	}, []);

	const getColumnSearchProps = dataIndex => ({
		filterDropdown: ({
			setSelectedKeys,
			selectedKeys,
			confirm,
			clearFilters
		}) => (
			<div style={{ padding: 8 }}>
				<Input
					placeholder={`Search ${dataIndex}`}
					value={selectedKeys[0]}
					onChange={e =>
						setSelectedKeys(e.target.value ? [e.target.value] : [])
					}
					onPressEnter={() => handleSearch(selectedKeys, confirm, dataIndex)}
					style={{ marginBottom: 8, display: "block" }}
				/>
				<Space>
					<Button
						type="primary"
						onClick={() => handleSearch(selectedKeys, confirm, dataIndex)}
						icon={<SearchOutlined />}
						size="small"
						style={{ width: 90 }}
					>
						Search
					</Button>
					<Button
						onClick={() => handleReset(clearFilters)}
						size="small"
						style={{ width: 90 }}
					>
						Reset
					</Button>
				</Space>
			</div>
		),
		filterIcon: filtered => (
			<SearchOutlined style={{ color: filtered ? "#1890ff" : undefined }} />
		),
		onFilter: (value, record) =>
			record[dataIndex].toString().toLowerCase().includes(value.toLowerCase())
	});

	const defaultColumns = [
		{
			title: "Product",
			dataIndex: "product",
			key: "product",
			...getColumnSearchProps("product"),
			sorter: (a, b) => a.product.localeCompare(b.product)
		},
		{
			title: "Category",
			dataIndex: "category",
			key: "category",
			filters: [
				{ text: "Electronics", value: "Electronics" },
				{ text: "Computers", value: "Computers" },
				{ text: "Accessories", value: "Accessories" },
				{ text: "Tablets", value: "Tablets" }
			],
			onFilter: (value, record) => record.category === value
		},
		{
			title: "Price",
			dataIndex: "price",
			key: "price",
			render: price => `$${price.toLocaleString()}`,
			sorter: (a, b) => a.price - b.price
		},
		{
			title: "Stock",
			dataIndex: "stock",
			key: "stock",
			sorter: (a, b) => a.stock - b.stock
		},
		{
			title: "Status",
			dataIndex: "status",
			key: "status",
			render: status => {
				const config = {
					available: { color: "green", text: "Available" },
					"low-stock": { color: "orange", text: "Low Stock" },
					"out-of-stock": { color: "red", text: "Out of Stock" }
				};
				const { color, text } = config[status] || {
					color: "default",
					text: status
				};
				return <Tag color={color}>{text}</Tag>;
			},
			filters: [
				{ text: "Available", value: "available" },
				{ text: "Low Stock", value: "low-stock" },
				{ text: "Out of Stock", value: "out-of-stock" }
			],
			onFilter: (value, record) => record.status === value
		},
		{
			title: "Last Updated",
			dataIndex: "lastUpdated",
			key: "lastUpdated",
			render: date => dayjs(date).format("MMM D, YYYY"),
			sorter: (a, b) =>
				dayjs(a.lastUpdated).unix() - dayjs(b.lastUpdated).unix()
		}
	];

	const columns = propColumns || defaultColumns;

	return (
		<Table
			columns={columns}
			dataSource={filteredData}
			pagination={{
				pageSize: 10,
				showSizeChanger: true,
				showTotal: total => `Total ${total} items`
			}}
			scroll={{ x: true }}
		/>
	);
};

export default DataTable;
