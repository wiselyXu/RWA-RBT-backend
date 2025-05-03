#!/bin/bash

# Install needed packages
echo "Installing needed packages..."
npm install http-proxy-middleware --save

# Start the development server
echo "Starting development server with proxy configuration..."
echo "API requests to /rwa/* will be forwarded to http://127.0.0.1:8888/rwa/*"
npm start 