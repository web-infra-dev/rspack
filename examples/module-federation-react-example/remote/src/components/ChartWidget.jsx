import {
	ArcElement,
	BarElement,
	CategoryScale,
	Chart as ChartJS,
	Legend,
	LinearScale,
	LineElement,
	PointElement,
	Title,
	Tooltip
} from "chart.js";
import { random } from "lodash-es";
import { useMemo } from "react";
import { Bar, Doughnut, Line, Pie } from "react-chartjs-2";

ChartJS.register(
	CategoryScale,
	LinearScale,
	PointElement,
	LineElement,
	BarElement,
	ArcElement,
	Title,
	Tooltip,
	Legend
);

const ChartWidget = ({
	type = "line",
	data: propData,
	options: propOptions,
	height = 300
}) => {
	const defaultData = useMemo(() => {
		const labels = [
			"January",
			"February",
			"March",
			"April",
			"May",
			"June",
			"July"
		];
		const dataset1 = labels.map(() => random(10, 100));
		const dataset2 = labels.map(() => random(10, 100));

		switch (type) {
			case "pie":
			case "doughnut":
				return {
					labels: ["Red", "Blue", "Yellow", "Green", "Purple"],
					datasets: [
						{
							data: [
								random(10, 50),
								random(10, 50),
								random(10, 50),
								random(10, 50),
								random(10, 50)
							],
							backgroundColor: [
								"rgba(255, 99, 132, 0.6)",
								"rgba(54, 162, 235, 0.6)",
								"rgba(255, 206, 86, 0.6)",
								"rgba(75, 192, 192, 0.6)",
								"rgba(153, 102, 255, 0.6)"
							],
							borderColor: [
								"rgba(255, 99, 132, 1)",
								"rgba(54, 162, 235, 1)",
								"rgba(255, 206, 86, 1)",
								"rgba(75, 192, 192, 1)",
								"rgba(153, 102, 255, 1)"
							],
							borderWidth: 1
						}
					]
				};
			default:
				return {
					labels,
					datasets: [
						{
							label: "Dataset 1",
							data: dataset1,
							borderColor: "rgb(255, 99, 132)",
							backgroundColor: "rgba(255, 99, 132, 0.5)",
							tension: 0.1
						},
						{
							label: "Dataset 2",
							data: dataset2,
							borderColor: "rgb(53, 162, 235)",
							backgroundColor: "rgba(53, 162, 235, 0.5)",
							tension: 0.1
						}
					]
				};
		}
	}, [type]);

	const defaultOptions = {
		responsive: true,
		maintainAspectRatio: false,
		plugins: {
			legend: {
				position: "top"
			},
			title: {
				display: true,
				text: `Sample ${type.charAt(0).toUpperCase() + type.slice(1)} Chart`
			}
		}
	};

	const chartData = propData || defaultData;
	const chartOptions = propOptions || defaultOptions;

	const renderChart = () => {
		const props = {
			data: chartData,
			options: chartOptions,
			height
		};

		switch (type) {
			case "line":
				return <Line {...props} />;
			case "bar":
				return <Bar {...props} />;
			case "pie":
				return <Pie {...props} />;
			case "doughnut":
				return <Doughnut {...props} />;
			default:
				return <Line {...props} />;
		}
	};

	return <div style={{ height, position: "relative" }}>{renderChart()}</div>;
};

export default ChartWidget;
