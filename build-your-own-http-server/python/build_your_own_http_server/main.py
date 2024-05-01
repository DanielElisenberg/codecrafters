import socket
import os
import threading
import argparse


def parse_request(request_string: str):
    req_start_line, request_string = request_string.split("\r\n", 1)
    if "\r\n\r\n" in request_string:
        request_headers, request_body = request_string.split("\r\n\r\n", 1)
    else:
        request_headers = request_string
        request_body = ""
    header_tuples = [
        header.split(": ") for header in request_headers.split("\r\n") if header
    ]
    return {
        "method": req_start_line.split(" ")[0],
        "path": req_start_line.split(" ")[1],
        "headers": {key: value for (key, value) in header_tuples},
        "body": request_body,
    }


def echo_path_handler(request_dict: dict) -> bytes:
    echo_string = request_dict["path"].replace("/echo/", "")
    return (
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: "
        + str(len(echo_string)).encode()
        + b"\r\n\r\n"
        + echo_string.encode()
    )


def echo_user_agent_handler(request_dict: dict) -> bytes:
    return (
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: "
        + str(len(request_dict["headers"]["User-Agent"])).encode()
        + b"\r\n\r\n"
        + request_dict["headers"]["User-Agent"].encode()
    )


def get_file_handler(request_dict: dict, config: argparse.Namespace) -> bytes:
    file_name = request_dict["path"].replace("/files/", "")
    if not os.path.isfile(os.path.join(config.directory, file_name)):
        return b"HTTP/1.1 404 Not Found\r\n\r\nNot Found"

    with open(os.path.join(config.directory, file_name), "rb") as f:
        file_data = f.read()
    return (
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: "
        + str(len(file_data)).encode()
        + b"\r\n\r\n"
        + file_data
    )


def post_file_handler(request_dict: dict, config: argparse.Namespace) -> bytes:
    file_name = request_dict["path"].replace("/files/", "")
    with open(os.path.join(config.directory, file_name), "wb") as f:
        f.write(request_dict["body"].encode())
    return b"HTTP/1.1 201 OK\r\n\r\nOK"


def handle_connection(connection: socket.socket, config: argparse.Namespace):
    request_dict = parse_request(connection.recv(1024).decode())
    if request_dict["path"].startswith("/files") and config.directory:
        if request_dict["method"] == "GET":
            connection.send(get_file_handler(request_dict, config))
        elif request_dict["method"] == "POST":
            connection.send(post_file_handler(request_dict, config))
        else:
            connection.send(
                b"HTTP/1.1 405 Method Not Allowed\r\n\r\nMethod Not Allowed"
            )
    elif request_dict["path"].startswith("/echo"):
        connection.send(echo_path_handler(request_dict))
    elif request_dict["path"] == "/user-agent":
        connection.send(echo_user_agent_handler(request_dict))
    elif request_dict["path"] == "/":
        connection.send(b"HTTP/1.1 200 OK\r\n\r\nHello, world!")
    else:
        connection.send(b"HTTP/1.1 404 Not Found\r\n\r\nNot Found")
    connection.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-d", "--directory", dest="directory", help="directory to serve files from"
    )
    args = parser.parse_args()
    server_socket = socket.create_server(("localhost", 4221), reuse_port=True)
    while True:
        connection, _ = server_socket.accept()
        threading.Thread(target=handle_connection, args=(connection, args)).start()


if __name__ == "__main__":
    main()
