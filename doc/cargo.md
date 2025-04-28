# 常用命令

#### 整体编译发布版
`cargo build --release --bin api`

## 测试
`cargo test --all -- --nocapture`


## 清理docker镜像
`docker rmi $(docker images | grep "none" | awk '{print $3}') `

#### 运行命令
`cargo run --bin opc-mqtt`

## 统计代码行数
`tokei ./crates --type=Rust`

## clickhouse
`docker run --name clickhouse bitnami/clickhouse:latest`

