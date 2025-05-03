const { createProxyMiddleware } = require('http-proxy-middleware');

module.exports = function(app) {
  app.use(
    '/rwa',
    createProxyMiddleware({
      //target: 'http://43.134.99.111:8888',
      target: 'http://127.0.0.1:8888',
      changeOrigin: true,
      pathRewrite: {
        '^/rwa': '/rwa', // Keep the /rwa prefix when forwarding requests
      },
      logLevel: 'debug', // Enable this for debugging
    })
  );
}; 