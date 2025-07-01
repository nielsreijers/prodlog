# Prodlog React UI

This is a modern React-based user interface for Prodlog, replacing the original server-side rendered HTML interface.

## Features

- **Modern React Interface**: Built with React 18, TypeScript, and React Router
- **Responsive Design**: Mobile-friendly layout that adapts to different screen sizes
- **Real-time Updates**: Seamless interaction with the existing Rust backend APIs
- **Enhanced UX**: Improved filtering, navigation, and user feedback
- **SPA Experience**: Single-page application with client-side routing

## Getting Started

### Prerequisites

- Node.js (v16 or higher)
- npm or yarn
- The Prodlog Rust backend must be running

### Installation and Build

1. **Install dependencies:**
   ```bash
   cd react-ui
   npm install
   ```

2. **Build the React app:**
   ```bash
   npm run build
   ```

3. **Or use the convenience script from the project root:**
   ```bash
   ./build-react.sh
   ```

### Development Mode

For development with hot reloading:

```bash
cd react-ui
npm run dev
```

This will start the development server on `http://localhost:5173` with proxy configuration to forward API requests to the Rust backend.

## Architecture

### Components

- **IndexPage**: Main listing page with filtering and entry table
- **EntryPage**: Detailed entry view with collapsible edit sections
- **RedactPage**: Bulk password redaction interface

### API Integration

The React app communicates with the existing Rust backend through these endpoints:

- `GET /api/entries` - List entries with filtering
- `GET /api/entry/:uuid` - Get single entry details
- `POST /api/entry` - Update entry (message, noop status)
- `POST /api/entry/redact` - Redact password from single entry
- `POST /api/redact` - Bulk redact passwords
- `GET /diffcontent/:uuid` - Get diff content for file changes

### Routing

The React app uses client-side routing:

- `/` - Main entry listing
- `/entry/:uuid` - Entry details
- `/redact` - Bulk password redaction

## Compatibility

The React UI is designed to be a drop-in replacement for the original server-side rendered interface. All existing functionality is preserved:

- Entry filtering and search
- Clickable table rows
- Copy-to-clipboard functionality
- Password redaction (single and bulk)
- Noop toggle with immediate updates
- Diff viewing for file changes
- Terminal output display

## Fallback

If the React build is not available, the server will show a fallback page with instructions. The original server-side rendered UI remains available at `/legacy/` routes.

## Browser Support

- Chrome/Edge 88+
- Firefox 85+
- Safari 14+

## Development

### File Structure

```
react-ui/
├── src/
│   ├── components/         # React components
│   │   ├── IndexPage.tsx   # Main listing page
│   │   ├── EntryPage.tsx   # Entry details page
│   │   └── RedactPage.tsx  # Bulk redaction page
│   ├── types.ts           # TypeScript type definitions
│   ├── api.ts             # API service layer
│   ├── styles.css         # CSS styles (ported from original)
│   └── main.tsx           # React app entry point
├── package.json
├── vite.config.ts         # Vite build configuration
└── tsconfig.json          # TypeScript configuration
```

### Build Process

1. Vite compiles TypeScript and bundles the React app
2. Output goes to `../src/ui/static/react/`
3. Rust server serves the built files and uses SPA fallback routing

### Adding New Features

1. Add new API endpoints in the Rust backend if needed
2. Update `src/types.ts` with new TypeScript interfaces
3. Extend `src/api.ts` with new API methods
4. Create or modify React components
5. Update routing in `src/main.tsx` if adding new pages

## Legacy UI

The original server-side rendered UI is still available at:

- `/legacy/` - Main listing
- `/legacy/entry/:uuid` - Entry details  
- `/legacy/redact` - Bulk redaction

This provides a fallback if the React build fails or for users who prefer the original interface. 