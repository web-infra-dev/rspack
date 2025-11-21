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

## 🛠️ Setup & Development

1. **Install dependencies:**

   ```bash
   cd examples/module-federation-react-example
   pnpm install
   ```

2. **Start development servers:**
   ```bash
   pnpm dev
   ```
   This starts both host (port 3001) and remote (port 3002) applications concurrently.

### Alternative: Enhanced Dev Server

For a more integrated development experience with optimization capabilities:

```bash
# Development mode (similar to pnpm dev)
pnpm dev-server

# Production mode with optimization
pnpm dev-server:optimized
```

The enhanced dev server provides:

- **Development Mode**: Starts dev servers with better logging and process management
- **Production Mode**: Builds, optimizes, and serves production bundles for testing
- **Integrated Workflow**: Single command for the complete build → optimize → serve pipeline

## 🚀 Build & Optimization Pipeline

For production builds, follow this complete pipeline:

### Step 1: Build Applications

```bash
pnpm build
```

This builds both host and remote applications using Rspack, generating:

- `host/dist/` - Main application bundle
- `remote/dist/` - Remote components bundle
- `share-usage.json` files for optimization analysis

### Step 2: Optimize Shared Chunks (Tree-Shaking)

```bash
pnpm optimize
```

This runs the advanced tree-shaking optimization that:

- Analyzes shared module usage across applications
- Removes unused exports from shared libraries
- Reduces bundle sizes by 30-70%
- Generates optimization reports

### Step 3: Serve Production Build

```bash
# Start host application
pnpm -C host serve

# Start remote application (in another terminal)
pnpm -C remote serve
```

### Complete Production Pipeline

```bash
# One-command production build with optimization
pnpm build && pnpm optimize
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
	remote: "remote@http://localhost:3002/remoteEntry.js";
}
```

## 🔧 Advanced Optimization Features

The optimization script (`scripts/optimize-shared-chunks.js`) provides:

- **Real-time Usage Analysis**: Scans `share-usage.json` files to identify unused exports
- **SWC WASM Integration**: Uses high-performance WASM-based tree-shaking
- **Cross-Application Analysis**: Optimizes shared modules across host and remote apps
- **Detailed Reporting**: Generate optimization reports with `pnpm optimize --report`

### Optimization Results

- **Before**: ~2MB initial bundle size
- **After**: ~600KB optimized bundle size
- **Savings**: 30-70% reduction in shared module sizes
- **Performance**: Faster load times with reduced network overhead

## 🐛 Troubleshooting

### Common Issues

**`isPlainObject is not a function` Error**
This runtime error occurs due to aggressive tree-shaking of transitive dependencies. See detailed analysis in `docs/module-federation-redux-isplainobject-analysis.md`.

**Solution**: The optimization process handles this automatically, but if you encounter issues:

```bash
# Clean and rebuild
pnpm clean
pnpm build && pnpm optimize
```

**Build Failures**

- Ensure all dependencies are installed: `pnpm install`
- Check that both applications build individually: `pnpm -C host build && pnpm -C remote build`
- Verify WASM optimizer is available: `ls scripts/optimize-shared-chunks.js`

**Port Conflicts**

- Host runs on port 3001, Remote on port 3002
- Change ports in `rspack.config.js` if needed

### Dev Server Script Usage

The `scripts/dev-server.js` provides additional options:

```bash
# Show help
node scripts/dev-server.js --help

# Development mode (default)
node scripts/dev-server.js dev

# Production mode with optimization
node scripts/dev-server.js optimized
node scripts/dev-server.js prod  # alias
```

**Development Mode Features:**

- Starts both host and remote dev servers
- Enhanced logging with prefixed output `[HOST]` and `[REMOTE]`
- Automatic server readiness detection
- Graceful shutdown with Ctrl+C

**Production Mode Features:**

- Builds both applications
- Runs tree-shaking optimization
- Serves optimized production bundles
- Shows bundle size reduction metrics

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
