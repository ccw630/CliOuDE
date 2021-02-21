# CliOuDE - Deploy
## Cluster(k8s)

- [x] Tested on Minikube
- [x] Tested on Aliyun Serverless Kubernetes

### Usage

```sh
cd k8s
kubectl apply -f nginx.yaml    # First create nginx config map
kubectl apply -f postgres.yaml # Tips: Create PV & PVC before start db
kubectl apply -f server.yaml   # Start server after db started. Worker & editor rely on this
kubectl apply -f lsphub.yaml
kubectl apply -f worker.yaml
kubectl apply -f editor.yaml   # Configure http:80 to provide Internet access on Aliyun
```