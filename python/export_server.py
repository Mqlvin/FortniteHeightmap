#!/usr/bin/env python3
import socket
import struct
import json
from threading import Thread
import sys
import signal
import time

import pakchunk_handler
import helpers

COMMAND_MESSAGE = 0
COMMAND_DATA = 1
HEADER_FMT = '=BI'
HEADER_SIZE = struct.calcsize(HEADER_FMT)

class Server(Thread):
    instance = None

    def __init__(self, host='127.0.0.1', port=40000):
        super().__init__(daemon=True)
        self.host = host
        self.port = port
        self.server = None
        self.running = False
        self.clients = []
        Server.instance = self

    @staticmethod
    def create():
        return Server()

    def run(self):
        print(f"Waiting for FortnitePorting on {self.host}:{self.port}")
        try:
            self.server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self.server.bind((self.host, self.port))
            self.server.listen(5)
            self.running = True

            while self.running:
                try:
                    client_socket, address = self.server.accept()
                    print(f"Received connection from FortnitePorting at {address}")
                    self.clients.append(client_socket)
                    client_thread = Thread(target=self.handle_client, args=(client_socket, address), daemon=True)
                    client_thread.start()
                except Exception as e:
                    if self.running:
                        print(f"[ERROR] Error accepting connection: {e}")
                    break

        except Exception as e:
            print(f"[ERROR] Server error: {e}")

    def recv_exact(self, sock, n):
        data = b''
        while len(data) < n:
            try:
                packet = sock.recv(n - len(data))
            except Exception as e:
                print(f"[ERROR] recv error: {e}")
                return None
            if not packet:
                return None
            data += packet
        return data

    def handle_client(self, client_socket, address):
        printed_dirs = False
        while True:
            header = self.recv_exact(client_socket, HEADER_SIZE)
            if not header:
                break
            command_type, data_size = struct.unpack(HEADER_FMT, header)
            payload = self.recv_exact(client_socket, data_size)
            if payload is None:
                break

            try:
                decoded_string = payload.decode('utf-8', errors='replace')
            except Exception:
                decoded_string = str(payload)

            if command_type == COMMAND_MESSAGE:
                print(f"[MESSAGE] {decoded_string}")
            elif command_type == COMMAND_DATA:
                parsed = json.loads(decoded_string)
                pakchunk_handler.handle_pakchunk(parsed, not printed_dirs)
                printed_dirs = True
            else:
                print(f"[UNKNOWN COMMAND {command_type}] Raw payload:")
                print(payload)

    def shutdown(self):
        print("Shutdown Server")
        self.running = False
        if self.server:
            try:
                self.server.close()
            except:
                pass
        for c in list(self.clients):
            try:
                c.close()
            except:
                pass
        self.clients.clear()

def main():
    helpers.create_export_folder()

    server = Server.create()
    server.start()

    def handle_sigint(signum, frame):
        print("\nCaught interrupt, shutting down...")
        server.shutdown()
        sys.exit(0)

    signal.signal(signal.SIGINT, handle_sigint)
    signal.signal(signal.SIGTERM, handle_sigint)

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        handle_sigint(None, None)

if __name__ == "__main__":
    main()
