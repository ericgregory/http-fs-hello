# HTTP Filesystem Demo

WIP

## Build the component

```console
```

> [!NOTE]
> While normally we might use something like `.cargo/config.toml` to set the target,
> we cannot 

## Testing

Start `kind` cluster from root of this directory with included kind config:

```shell
kind create cluster --config=kind-config.yaml
```

Install wasmCloud operator:

```shell
helm install wasmcloud --version 0.1.0 oci://ghcr.io/wasmcloud/charts/runtime-operator -f https://raw.githubusercontent.com/wasmCloud/wash/refs/heads/main/charts/runtime-operator/values.local.yaml
```

Update public-ingress host deployment with volume:

```shell
kubectl apply -f hostgroup.yaml
```

Deploy workload:

```shell
kubectl apply -f workloaddeployment.yaml
```

Test:

```shell
curl localhost/read-file -i
```
