#!/bin/bash

# Build React UI script
set -e

echo "Building React UI..."

# Check if node_modules exists in react-ui directory
if [ ! -d "react-ui/node_modules" ]; then
    echo "Installing React dependencies..."
    cd react-ui
    npm install
    cd ..
fi

# Build the React app
echo "Building React app..."
cd react-ui
npm run build
cd ..

# Create necessary directory structure in src/ui/static if it doesn't exist
mkdir -p src/ui/static/react

echo "React build complete. Files are in src/ui/static/react/"
echo "You can now run your Rust server to serve the React app." 