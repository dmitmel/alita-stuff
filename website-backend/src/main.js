const http = require('http');

let server = http.createServer((_req, res) => {
  res.end('Hello world!');
});

server.listen(8080);
