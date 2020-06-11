# CLIOUDE - Deploy

## 单机部署(docker-compose)

同时启动 Editor、Server、Worker 及数据库存储(PostgreSQL)容器。

### 依赖

需要安装 Docker 及 docker-compose。

### 使用方式
```sh
docker-compose pull
docker-compose up -d
```

可以通过修改`docker-compose.yaml`调整 Server 和 Worker 的实例数。注意增加 Server 时，需要同时在`nginx/default.conf`增加对应的 upstream server


## 分布式部署(k8s)

### Tested on Aliyun Serverless Kubernetes

### 部署顺序
```sh
kubectl create -f nginx.yaml    # nginx config map 配置
kubectl create -f postgres.yaml # 起 DB 前，先挂载云盘，建立存储声明；server 启动执行 alembic 依赖 DB
kubectl create -f server.yaml   # worker heartbeat 依赖 server；editor nginx 代理依赖 server 地址
kubectl create -f lsphub.yaml   # LSP Hub，无其他依赖
kubectl create -f worker.yaml   # 可以配置多个 service，比如 worker1, worker2, worker3
kubectl create -f editor.yaml   # editor service 启动后配置 80 端口映射，以提供外网访问
```