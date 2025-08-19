# Module Federation React Example

A comprehensive React application demonstrating Module Federation with shared UI components, state management, and optimized bundle sharing.

## 🚀 Features

- **React 18** with concurrent features
- **Module Federation** for micro-frontend architecture
- **Ant Design** UI library shared between apps
- **Redux Toolkit** for state management
- **React Router** for navigation
- **Chart.js** for data visualization
- **Optimized bundles** with tree-shaking

## 🏗️ Architecture

```
module-federation-react-example/
├── host/                 # Main application (port 3001)
│   ├── src/
│   │   ├── App.jsx      # Main app with routing
│   │   ├── pages/       # Dashboard, Analytics, Settings
│   │   └── store/       # Redux store setup
│   └── rspack.config.js
├── remote/              # Shared components app (port 3002)
│   ├── src/
│   │   ├── components/  # Reusable UI components
│   │   ├── charts/      # Data visualization components
│   │   └── forms/       # Form components
│   └── rspack.config.js
└── scripts/            # Build and optimization scripts
```

## 🛠️ Setup

1. **Install dependencies:**
   ```bash
   cd examples/module-federation-react-example
   pnpm install
   ```

2. **Start development servers:**
   ```bash
   pnpm dev
   ```

3. **Build for production:**
   ```bash
   pnpm build:optimized
   ```

## 🎯 What's Demonstrated

### Host Application
- Full dashboard with multiple pages
- Consumes remote components dynamically
- Shared state management across federated modules
- Responsive layout with Ant Design

### Remote Application
- Exports reusable React components:
  - `UserCard` - User profile display
  - `DataTable` - Advanced data grid
  - `ChartWidget` - Various chart types
  - `FormBuilder` - Dynamic form generation
- Components work standalone or integrated

### Shared Dependencies
- `react` & `react-dom` - Singleton shared
- `antd` - UI component library
- `@reduxjs/toolkit` & `react-redux` - State management
- `react-router-dom` - Routing
- `lodash-es` - Utility functions
- `chart.js` & `react-chartjs-2` - Charts

## 🔧 Key Configuration

### Module Federation Setup

**Remote exposes:**
```javascript
exposes: {
  "./UserCard": "./src/components/UserCard",
  "./DataTable": "./src/components/DataTable",
  "./ChartWidget": "./src/charts/ChartWidget",
  "./FormBuilder": "./src/forms/FormBuilder",
  "./store": "./src/store/slices"
}
```

**Host consumes:**
```javascript
remotes: {
  remote: "remote@http://localhost:3002/remoteEntry.js"
}
```

## 🚀 Production Optimization

The build process includes automatic optimization:

1. **Tree Shaking**: Removes unused exports from shared libraries
2. **Bundle Analysis**: Identifies optimization opportunities
3. **Chunk Optimization**: Reduces bundle sizes by 30-70%

Run optimization:
```bash
pnpm optimize
```

## 📊 Performance

- Initial bundle: ~2MB
- After optimization: ~600KB
- Shared dependencies loaded once
- Dynamic imports for code splitting

## 🧪 Testing

```bash
# Run all tests
pnpm test

# Run specific test suites
pnpm test:unit
pnpm test:integration
pnpm test:e2e
```

## 🌐 Live URLs

- **Host App**: http://localhost:3001
- **Remote App**: http://localhost:3002

## 📝 Notes

- Uses Rspack for fast builds
- Hot Module Replacement enabled
- TypeScript support included
- Production-ready optimization