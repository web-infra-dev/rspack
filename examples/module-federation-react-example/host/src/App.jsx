import { Layout, Spin } from "antd";
import { Suspense } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import AppHeader from "./components/AppHeader";
import AppSidebar from "./components/AppSidebar";
import Analytics from "./pages/Analytics";
import Dashboard from "./pages/Dashboard";
import RemoteShowcase from "./pages/RemoteShowcase";
import Settings from "./pages/Settings";
import Users from "./pages/Users";

const { Content } = Layout;

const LoadingFallback = () => (
	<div className="loading-container">
		<Spin size="large" tip="Loading..." />
	</div>
);

function App() {
	return (
		<BrowserRouter>
			<Layout className="app-layout">
				<AppSidebar />
				<Layout>
					<AppHeader />
					<Content className="app-content">
						<Suspense fallback={<LoadingFallback />}>
							<Routes>
								<Route
									path="/"
									element={<Navigate to="/dashboard" replace />}
								/>
								<Route path="/dashboard" element={<Dashboard />} />
								<Route path="/analytics" element={<Analytics />} />
								<Route path="/users" element={<Users />} />
								<Route path="/remote-components" element={<RemoteShowcase />} />
								<Route path="/settings" element={<Settings />} />
							</Routes>
						</Suspense>
					</Content>
				</Layout>
			</Layout>
		</BrowserRouter>
	);
}

export default App;
