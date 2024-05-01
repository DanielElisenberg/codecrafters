package main

import (
	"flag"
	"fmt"
	"net"
	"os"
	"strings"
)

type Request struct {
	Method  string
	Path    string
	Headers map[string]string
	Body    string
}

func parseRequest(reqStr string) (r *Request) {
	startLine := strings.Split(reqStr, "\r\n")[0]
	method := strings.Split(startLine, " ")[0]
	path := strings.Split(startLine, " ")[1]
	headers_map := make(map[string]string)
	splitReq := strings.Split(reqStr, "\r\n\r\n")
	headers := strings.Split(splitReq[0], "\r\n")[1:]
	for _, header := range headers {
		if !strings.Contains(header, ": ") || strings.TrimSpace(header) == "" {
			continue
		}
		header_split := strings.Split(header, ": ")
		headers_map[header_split[0]] = header_split[1]
	}
	body := ""
	if len(splitReq) > 1 {
		body = splitReq[1]
	}
	return &Request{
		Method:  method,
		Path:    path,
		Headers: headers_map,
		Body:    body,
	}
}

func connectionHandler(conn net.Conn, directory string) {
	bytes := make([]byte, 1024)
	_, err := conn.Read(bytes)
	if err != nil {
		fmt.Println("Error reading: ", err.Error())
		os.Exit(1)
	}
	request := parseRequest(string(bytes))
	if request.Path == "/" {
		conn.Write([]byte("HTTP/1.1 200 OK\r\n\r\n"))
	} else if strings.HasPrefix(request.Path, "/echo") {
		echo := strings.TrimPrefix(request.Path, "/echo/")
		conn.Write(
			[]byte("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: " +
				fmt.Sprint(len(echo)) +
				"\r\n\r\n" + echo))
	} else if request.Path == "/user-agent" {
		conn.Write(
			[]byte("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: " +
				fmt.Sprint(len(request.Headers["User-Agent"])) +
				"\r\n\r\n" +
				request.Headers["User-Agent"]))
	} else if strings.HasPrefix(request.Path, "/files") {
		if directory == "" {
			conn.Write([]byte("HTTP/1.1 500 Internal Server Error\r\n\r\n"))
			conn.Close()
			return
		}
		if request.Method == "GET" {
			filename := strings.TrimPrefix(request.Path, "/files/")
			filePath := directory + filename
			if _, err := os.Stat(filePath); os.IsNotExist(err) {
				conn.Write([]byte("HTTP/1.1 404 Not Found\r\n\r\n"))
				conn.Close()
				return
			}
			file, err := os.ReadFile(filePath)
			if err != nil {
				conn.Write([]byte("HTTP/1.1 500 Internal Server Error\r\n\r\n"))
				conn.Close()
				return
			}
			conn.Write([]byte(
				"HTTP/1.1 200 OK\r\n" +
					"Content-Type: application/octet-stream\r\n" +
					"Content-Length: " + fmt.Sprint(len(file)) +
					"\r\n\r\n",
			))
			conn.Write(file)
		} else if request.Method == "POST" {
			filename := strings.TrimPrefix(request.Path, "/files/")
			filePath := directory + filename
			file, err := os.Create(filePath)
			if err != nil {
				conn.Write([]byte("HTTP/1.1 500 Internal Server Error\r\n\r\n"))
				conn.Close()
				return
			}
			_, err = file.Write([]byte(strings.ReplaceAll(request.Body, "\x00", "")))
			if err != nil {
				conn.Write([]byte("HTTP/1.1 500 Internal Server Error\r\n\r\n"))
				conn.Close()
				return
			}
			file.Close()
			conn.Write([]byte("HTTP/1.1 201 OK\r\n\r\n"))
		} else {
			conn.Write([]byte("HTTP/1.1 405 Method Not Allowed\r\n\r\n"))
		}
	} else {
		conn.Write([]byte("HTTP/1.1 404 Not Found\r\n\r\n"))
	}
	conn.Close()
}

func main() {
	var directory string
	flag.StringVar(&directory, "directory", "", "Directory to serve")
	flag.Parse()

	l, err := net.Listen("tcp", "0.0.0.0:4221")
	if err != nil {
		fmt.Println("Failed to bind to port 4221")
		os.Exit(1)
	}
	for {
		conn, err := l.Accept()
		if err != nil {
			fmt.Println("Error accepting connection: ", err.Error())
			os.Exit(1)
		}
		go connectionHandler(conn, directory)
	}
}
