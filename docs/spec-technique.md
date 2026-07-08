# Spec technique

## Structure

- `.gitlab-ci.yml` inclut les components `shared-ci/ci-templates` à la ref
  `v2.0.0`.
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

## Inputs CI

Le dépôt fournit les inputs principaux suivants aux components `ci-templates` :

- `app_name=helloworld` ;
- `dockerfile`/`context_path`/`snapshot_image`/`release_image` par service,
  passés au component `build-docker` (un jeu par service, fan-out via
  `parallel: matrix:` pour le second service — voir `.gitlab-ci.yml`) ;
- `service_name=helloworld-gui`, service vitrine pour les URLs GitLab ;
- `manifests_project_path=hello-groupe/helloworld-iac` ;
- `manifests_path=k8s` ;
- `has_preprod=true`.

## Contrat avec la plateforme

Chaque service doit conserver un sous-dossier portant son nom et un
`Dockerfile` à sa racine. Le component `build-docker` (qui enrobe
to-be-continuous/docker) utilise ce chemin pour construire les images.

Le déploiement est indirect : les jobs CI clonent `helloworld-iac`, modifient
`k8s/kustomization.yaml`, puis poussent sur la branche d'environnement
appropriée. ArgoCD synchronise ensuite le cluster.
