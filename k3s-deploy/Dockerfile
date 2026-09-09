# RGS 5 域 docker image (per 9/9 16:00 JST 拍板 A 起 k3s 部署)
# 简化版: 复用已 release build 的 5 binary, 不用重 build
# Base: mcr.microsoft.com/windows/servercore:ltsc2022 (Windows 容器, 兼容 native .exe)
# 真实 k8s 部署: 见 docs/deploy/01-k8s-manifests/01-player-service.yaml (5 域)

FROM mcr.microsoft.com/windows/servercore:ltsc2022

WORKDIR C:/app
COPY bin/ C:/app/bin/

# 5 binary 共用 image, 用 ENTRYPOINT 选择
# docker run rgs:<tag> player-service  (或 economy/match/social/admin)
ENTRYPOINT ["C:/app/bin/admin-service.exe"]
