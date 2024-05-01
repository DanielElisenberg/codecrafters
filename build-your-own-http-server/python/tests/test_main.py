import requests


HTTP_SERVER_URL = "http://localhost:4221"


def test_request_get():
    res = requests.get(HTTP_SERVER_URL)
    assert res.status_code == 200
    assert res.text == "Hello, world!"


if __name__ == "__main__":
    test_request_get()
