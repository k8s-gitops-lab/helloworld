Place the Zscaler root CA certificate here as a `.crt` file, for example:

```text
helloworld-svc/certs/zscaler-root-ca.crt
```

The service Dockerfile copies this directory into both the Rust build image and
the runtime image, then runs `update-ca-certificates`.
