# css-inline contribution guidelines

Thank you for your interest in making `css-inline` better!
We'd love to have your contribution. We expect all contributors to
abide by the Contributor Covenant Code of Conduct, which you can find in the
[`CODE_OF_CONDUCT.md`] file in this repository.

[`CODE_OF_CONDUCT.md`]: https://github.com/Stranger6667/css-inline/blob/master/CODE_OF_CONDUCT.md

## License

The code in this project is licensed under MIT license.
By contributing to `css-inline`, you agree that your contributions will be licensed under its MIT license.

## Pull Requests

To make changes to `css-inline`, please send in pull requests on GitHub to the `master`
branch. We'll review them and either merge or request changes. Github Actions test
everything as well, so you may get feedback from it too.

If you make additions or other changes to a pull request, feel free to either amend
previous commits or only add new ones, however you prefer. We may ask you to squash
your commits before merging, depending.

## Feature requests and feedback

If you'd like to suggest a feature, feel free to [submit an issue](https://github.com/Stranger6667/css-inline/issues)
and:

- Write a simple and descriptive title to identify your suggestion.
- Provide as many details as possible, explain your context, and how the feature should work.
- Explain why this improvement would be useful.
- Keep the scope narrow. It will make it easier to implement.

## Report bugs

Report bugs for `css-inline` in the [issue tracker](https://github.com/Stranger6667/css-inline/issues).

If you are reporting a bug, please:

- Write a simple and descriptive title to identify the problem.
- Describe the exact steps which reproduce the problem in as many details as possible.
- Describe the behavior you observed after following the steps and point out the problem with that behavior.
- Explain which behavior you expected to see instead and why.
- Include your Rust and `css-inline` version. Additionally include your Python version if you use our Python bindings.

It would be awesome if you can submit a failing test that demonstrates the problem.

## Running tests

Running tests requires a test server running in background:

```
python ./css-inline/tests/server.py &
```

Then run tests inside the `css-inline` directory:

```
cargo test
```

## Preferred communication language

We prefer to keep all communications in English.
