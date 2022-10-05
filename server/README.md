# Lexica Inkplate Server

## Server

### Build multi-arch docker release

https://cloudolife.com/2022/03/05/Infrastructure-as-Code-IaC/Container/Docker/Docker-buildx-support-multiple-architectures-images/

```
docker buildx build \
  --push \
  --platform linux/arm/v7,linux/arm64/v8 \
  --tag jakobwesthoff/lexica-inkplate-server:0.1.0 .
```
