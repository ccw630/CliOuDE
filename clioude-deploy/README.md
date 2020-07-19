# CliOuDE - Deploy

## Standalone(docker-compose)

Start Editor, Server, Worker, LSP Hub & Database(PostgreSQL) containers at once.

### Dependency

* Docker
* docker-compose

### Usage
```sh
cd docker
docker-compose pull
docker-compose up -d
```

If you need more Server/Worker/LSP Hub instances, edit `docker-compose.yaml`, and don't forget to modify upstream servers in `nginx/default.conf`!


## Cluster(k8s)

- [x] Tested on Aliyun Serverless Kubernetes

### Usage

```sh
cd k8s
kubectl create -f nginx.yaml    # First create nginx config map
kubectl create -f postgres.yaml # Tips: Create PV & PVC before start db
kubectl create -f server.yaml   # Start server after db started. Worker & editor rely on this
kubectl create -f lsphub.yaml   # LSP Hub has no special dependencies
kubectl create -f worker.yaml   # You can create multiple services named by worker1, worker2, worker3..
kubectl create -f editor.yaml   # Configure http:80 to provide Internet access
```