# x-parser: X/Twitter post parsing tool written in Rust 

For the tool, use /parser.

For the API, use /parserapi.

## How to run the tool?

```
git clone https://github.com/untreu2/x-parser.git
cd x-parser
cd parser
cargo run --release
```

## How to run the API?

In parserapi/config.toml, you can define the server address.

(Default: 127.0.0.1:8080)

```
git clone https://github.com/untreu2/x-parser.git
cd x-parser
cd parserapi
cargo run --release
```

An example of a curl call to the API:
```
curl -G "http://127.0.0.1:8080/tweet_url" --data-urlencode "tweet_url=https://twitter.com/example/status/1234567890"
```
