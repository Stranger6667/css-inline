from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/external.css":
            self.send_response(200)
            self.send_header("Content-Type", "text/css")
            self.end_headers()
            self.wfile.write(b"h1 { color: blue; }")
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass  # Suppress logging


if __name__ == "__main__":
    server = ThreadingHTTPServer(("127.0.0.1", 1234), Handler)
    server.serve_forever()
