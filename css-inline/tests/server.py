from flask import Flask

app = Flask(__name__)


@app.route("/external.css")
def stylesheet():
    return "h1 { color: blue; }"


if __name__ == "__main__":
    app.run()
