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

### TODO