# ratatui-sample

## debug

```sh
# テスト用 API の起動
docker run --rm -it -p 5000:4010 -v $PWD:/tmp stoplight/prism:4 mock -h 0.0.0.0 /tmp/src/openstack/openapi.yaml
```
```sh
cargo run
```

## test

```sh
# テスト用 API の起動
docker run --rm -it -p 5000:4010 -v $PWD:/tmp stoplight/prism:4 mock -h 0.0.0.0 /tmp/src/openstack/openapi.yaml
```
```sh
cargo test
```
