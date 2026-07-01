# Spec technique

## Structure

- `.gitlab-ci.yml` inclut `infra/ci-templates` à la ref `v1.13.1`.
- `.releaserc.json` configure `semantic-release`.
- `docker-compose.yml` lance les deux services localement.
- `helloworld-svc/` contient l'API Rust Axum.
- `helloworld-gui/` contient un frontend statique servi par nginx.

## Backend

`helloworld-svc` est une API Rust basée sur Axum et Tokio. Elle écoute sur le
port `8000` dans le conteneur et expose les routes `/`, `/hello/:name` et
`/health`.

Le Dockerfile construit l'application Rust puis produit une image runtime. Les
certificats présents dans `helloworld-svc/certs/*.crt` sont installés pendant
le build et dans l'image finale pour supporter l'environnement réseau du POC.

## Frontend

`helloworld-gui` est un site statique servi par nginx sur le port `80`. En
local, Docker Compose l'expose sur `8080`. Dans Kubernetes, il est exposé via
les manifests du dépôt `helloworld-iac`.

## Variables CI

Le dépôt définit les variables principales suivantes :

- `APP_NAME=helloworld` ;
- `SERVICES`, liste des couples `<service>=<image>` ;
- `SERVICE_NAME=helloworld-gui`, service vitrine pour les URLs GitLab ;
- `MANIFESTS_PROJECT_PATH=infra/helloworld-iac` ;
- `MANIFESTS_PATH=k8s` ;
- `HAS_PREPROD=true`.

## Contrat avec la plateforme

Chaque service doit conserver un sous-dossier portant son nom et un
`Dockerfile` à sa racine. Le template CI utilise ce nom pour construire les
images avec Kaniko.

Le déploiement est indirect : les jobs CI clonent `helloworld-iac`, modifient
`k8s/kustomization.yaml`, puis poussent sur la branche d'environnement
appropriée. ArgoCD synchronise ensuite le cluster.
