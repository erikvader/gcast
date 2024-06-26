import sys
from http.server import SimpleHTTPRequestHandler
from socketserver import TCPServer

class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory="web-root", **kwargs)
        self.path = None # NOTE: just to get rid of pylint warning

    def end_headers(self):
        self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")
        super().end_headers()

    def do_GET(self):
        orgpath = self.path
        if orgpath.startswith("/debug/"):
            print("loading something in /debug, will use / instead")
            self.path = "/"
        super().do_GET()
        self.path = orgpath

class MyTcpServer(TCPServer):
    def server_bind(self):
        import socket
        self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        super().server_bind()

def main():
    if len(sys.argv) == 2:
        port = int(sys.argv[1])
    else:
        port = 8080

    with MyTcpServer(("", port), Handler) as httpd:
        print(f"Serving on {port}...")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("Exiting, bye bye")

if __name__ == "__main__":
    main()
