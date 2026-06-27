# helloworld

Application exemple du POC, sous forme de monorepo multi-services :

- `helloworld-svc` : API Rust (`GET /`, `/hello/{name}`, `/health`)
- `helloworld-gui` : frontend statique nginx

La CI est gérée par le template partagé `root/ci-templates` inclus dans `.gitlab-ci.yml`.

---

## Développement local

### Lancer l'application

```bash
docker compose up --build
# GUI  → http://localhost:8080
# API  → http://localhost:8081
```

### Certificat Zscaler pour l'image Rust

Le build Docker de `helloworld-svc` installe les certificats présents dans
`helloworld-svc/certs/*.crt` dans le build stage Rust et dans l'image runtime.
Le certificat racine Zscaler utilisé ici est `helloworld-svc/certs/zscaler-root-ca.crt`.

---

## CI locale avec gitlab-ci-local

[gitlab-ci-local](https://github.com/firecow/gitlab-ci-local) exécute les jobs GitLab CI
sur la machine de développement sans avoir à pousser sur GitLab.

### Prérequis

```bash
# macOS
brew install gitlab-ci-local

# Le dépôt poc-devops-ci-templates doit être un dossier voisin
ls ../poc-devops-ci-templates/scripts/deploy.py   # doit exister
```

### Configuration

`.gitlab-ci-local.yml` (commité dans ce dépôt) définit les variables locales et
remplace l'URL in-cluster de GitLab par l'adresse externe :

```yaml
variables:
  CI_SCRIPTS_DIR: /ci-scripts
  INTERNAL_GITLAB_HOST: gitlab.192.168.33.100.nip.io
```

Les jobs qui exécutent les scripts Python du template doivent monter le dépôt
voisin avec l'option `--volume ../poc-devops-ci-templates:/ci-scripts:ro`.

Les **secrets** vont dans un fichier gitignore à créer une seule fois :

```bash
cat > .gitlab-ci-local-secrets.yml << 'EOF'
variables:
  GITLAB_PUSH_TOKEN: <token-push-gitlab>
EOF
```

Le token doit avoir le scope `write_repository` sur `root/helloworld-iac`.
Il peut être créé dans GitLab → User Settings → Access Tokens.

Si `gitlab-ci-local` tente de récupérer `root/ci-templates` depuis GitHub,
précharger son cache d'includes depuis le dépôt voisin :

```bash
mkdir -p .gitlab-ci-local/includes/github.com/root/ci-templates/v0.11.0
cp ../poc-devops-ci-templates/gitlab-ci.yml \
  .gitlab-ci-local/includes/github.com/root/ci-templates/v0.11.0/gitlab-ci.yml
```

### Jobs disponibles

```bash
gitlab-ci-local --list
```

```
semantic-release  (prepare)   manuel
build-dev         (build)      main
build-rec         (build)      tag vX.Y.Z
deploy-dev        (deploy)     main
deploy-rec        (deploy)     tag vX.Y.Z
deploy-preprod    (deploy)     tag vX.Y.Z, manuel
deploy-prod       (deploy)     tag vX.Y.Z, manuel
rollback-prod     (promote)    manuel + REVERT_SHA
```

### Exécution

Les jobs de **déploiement** fonctionnent directement en local : ils clonent
`root/helloworld-iac`, mettent à jour `kustomization.yaml` via PyYAML et poussent.

```bash
# Déployer la version "local" sur dev (utilise CI_COMMIT_SHORT_SHA=local)
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  --volume ../poc-devops-ci-templates:/ci-scripts:ro \
  deploy-dev

# Déployer un tag de release sur rec
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  --volume ../poc-devops-ci-templates:/ci-scripts:ro \
  --variable CI_COMMIT_TAG=v1.2.3 deploy-rec

# Rollback prod (annule un commit sur la branche main d'helloworld-iac)
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  --volume ../poc-devops-ci-templates:/ci-scripts:ro \
  --variable REVERT_SHA=abc1234 rollback-prod
```

### Jobs de build (Kaniko)

Les jobs `build-dev` et `build-rec` utilisent Kaniko, qui construit les images
**sans démon Docker** et les pousse directement dans le registry.

En local, Kaniko a besoin d'accéder au registry sur le réseau du cluster :

```bash
# Vérifier que le registry est joignable
curl http://registry.192.168.33.100.nip.io/v2/

# Puis lancer le build (pousse vers le registry local)
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  --variable SERVICES="helloworld-svc=registry.192.168.33.100.nip.io/helloworld-svc helloworld-gui=registry.192.168.33.100.nip.io/helloworld-gui" \
  build-dev
```

> **Note ARM64** : L'image Kaniko (`gcr.io/kaniko-project/executor:debug`) est amd64.
> Sur Apple Silicon, elle s'exécute via la couche d'émulation Rosetta de Docker Desktop.
> Pour des builds locaux rapides, préférer `docker compose build`.

---

## Pipeline CI/CD complet

La chaîne de promotion est déclenchée depuis GitLab (push sur `main` ou tag `vX.Y.Z`) :

```
push main
  └─ build-dev  ──►  deploy-dev   (automatique)
     tag vX.Y.Z
  └─ build-rec  ──►  deploy-rec   (automatique)
                 ──►  deploy-preprod (manuel)
                 ──►  deploy-prod    (manuel)
```

Le tag est créé manuellement via le job `semantic-release` (analyse les
Conventional Commits depuis le dernier tag et pousse un tag `vX.Y.Z`).
