import { ReloadOutlined, SaveOutlined } from "@ant-design/icons";
import {
	Button,
	Checkbox,
	DatePicker,
	Form,
	Input,
	Radio,
	Select,
	Space
} from "antd";
import { capitalize } from "lodash-es";

const FormBuilder = ({
	fields = [],
	onSubmit,
	initialValues = {},
	layout = "vertical"
}) => {
	const [form] = Form.useForm();

	const handleSubmit = values => {
		if (onSubmit) {
			onSubmit(values);
		}
	};

	const handleReset = () => {
		form.resetFields();
	};

	const renderField = field => {
		const {
			type,
			name,
			label,
			placeholder,
			required = false,
			rules = [],
			options = [],
			...restProps
		} = field;

		const baseRules = [
			...(required
				? [{ required: true, message: `Please provide ${label}` }]
				: []),
			...rules
		];

		switch (type) {
			case "text":
			case "email":
			case "url":
				return (
					<Form.Item
						key={name}
						name={name}
						label={label}
						rules={[
							...baseRules,
							...(type === "email"
								? [{ type: "email", message: "Please enter a valid email" }]
								: []),
							...(type === "url"
								? [{ type: "url", message: "Please enter a valid URL" }]
								: [])
						]}
					>
						<Input
							type={type}
							placeholder={placeholder || `Enter ${label}`}
							{...restProps}
						/>
					</Form.Item>
				);

			case "password":
				return (
					<Form.Item key={name} name={name} label={label} rules={baseRules}>
						<Input.Password
							placeholder={placeholder || `Enter ${label}`}
							{...restProps}
						/>
					</Form.Item>
				);

			case "textarea":
				return (
					<Form.Item key={name} name={name} label={label} rules={baseRules}>
						<Input.TextArea
							rows={4}
							placeholder={placeholder || `Enter ${label}`}
							{...restProps}
						/>
					</Form.Item>
				);

			case "select":
				return (
					<Form.Item key={name} name={name} label={label} rules={baseRules}>
						<Select
							placeholder={placeholder || `Select ${label}`}
							{...restProps}
						>
							{options.map(option => (
								<Select.Option
									key={typeof option === "string" ? option : option.value}
									value={typeof option === "string" ? option : option.value}
								>
									{typeof option === "string"
										? capitalize(option)
										: option.label}
								</Select.Option>
							))}
						</Select>
					</Form.Item>
				);

			case "date":
				return (
					<Form.Item key={name} name={name} label={label} rules={baseRules}>
						<DatePicker
							style={{ width: "100%" }}
							placeholder={placeholder || `Select ${label}`}
							{...restProps}
						/>
					</Form.Item>
				);

			case "checkbox":
				return (
					<Form.Item key={name} name={name} valuePropName="checked">
						<Checkbox {...restProps}>{label}</Checkbox>
					</Form.Item>
				);

			case "radio":
				return (
					<Form.Item key={name} name={name} label={label} rules={baseRules}>
						<Radio.Group {...restProps}>
							{options.map(option => (
								<Radio
									key={typeof option === "string" ? option : option.value}
									value={typeof option === "string" ? option : option.value}
								>
									{typeof option === "string"
										? capitalize(option)
										: option.label}
								</Radio>
							))}
						</Radio.Group>
					</Form.Item>
				);

			default:
				return null;
		}
	};

	return (
		<Form
			form={form}
			layout={layout}
			onFinish={handleSubmit}
			initialValues={initialValues}
			autoComplete="off"
		>
			{fields.map(renderField)}

			<Form.Item>
				<Space>
					<Button type="primary" htmlType="submit" icon={<SaveOutlined />}>
						Submit
					</Button>
					<Button
						htmlType="button"
						onClick={handleReset}
						icon={<ReloadOutlined />}
					>
						Reset
					</Button>
				</Space>
			</Form.Item>
		</Form>
	);
};

export default FormBuilder;
