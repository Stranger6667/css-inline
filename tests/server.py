from flask import Flask

app = Flask(__name__)

with open("tests/external.css") as fd:
    STYLESHEET = fd.read()


@app.route("/external.css")
def stylesheet():
    return STYLESHEET


if __name__ == "__main__":
    app.run()
