# ratatui-sample

OpenStack TUI Client using Ratatui

```sh
# start prisma for debug or test
docker run --rm -it -p 5000:4010 -v $PWD:/tmp stoplight/prism:4 mock -h 0.0.0.0 /tmp/src/openstack/openapi.yaml
```

```sh
cargo run

# run test
cargo test
```
