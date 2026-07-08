# helloworld

Application exemple du POC, sous forme de monorepo multi-services :

- `helloworld-svc` : API Rust (`GET /`, `/hello/{name}`, `/health`)
- `helloworld-gui` : frontend statique nginx

La CI est gérée par les components partagés `shared-ci/ci-templates` inclus dans `.gitlab-ci.yml`.

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

# Le dépôt ci-templates doit être un dossier voisin
ls ../ci-templates/scripts/deploy.py   # doit exister
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
voisin avec l'option `--volume ../ci-templates:/ci-scripts:ro`.

Les **secrets** vont dans un fichier gitignore à créer une seule fois :

```bash
cat > .gitlab-ci-local-secrets.yml << 'EOF'
variables:
  GITLAB_PUSH_TOKEN: <token-push-gitlab>
EOF
```

Le token doit avoir le scope `write_repository` sur `hello-groupe/helloworld-iac`.
Il peut être créé dans GitLab → User Settings → Access Tokens.

Si `gitlab-ci-local` tente de récupérer `shared-ci/ci-templates` depuis GitHub,
précharger son cache de components depuis le dépôt voisin :

```bash
# <ref> = la ref des components déclarée dans .gitlab-ci.yml (ex. v3.0.0)
mkdir -p .gitlab-ci-local/includes/github.com/shared-ci/ci-templates/<ref>/templates
cp -R ../ci-templates/templates/* \
  .gitlab-ci-local/includes/github.com/shared-ci/ci-templates/<ref>/templates/
```

### Jobs disponibles

```bash
gitlab-ci-local --list
```

```
semantic-release        (publish)        main
docker-buildah-build    (package-build)  main
docker-publish          (publish)        tag vX.Y.Z
deploy-dev              (deploy)         main
deploy-rec              (deploy)         tag vX.Y.Z
deploy-preprod          (deploy)         tag vX.Y.Z, manuel
deploy-prod             (deploy)         tag vX.Y.Z, manuel
rollback-prod           (promote)        manuel + REVERT_SHA
```

`docker-buildah-build`/`docker-publish` sont matrixés (un par service, voir
`.gitlab-ci.yml`) : `gitlab-ci-local --list` affiche donc en réalité une
entrée par service (ex. `docker-buildah-build: [helloworld-svc]`).

### Exécution

Les jobs de **déploiement** fonctionnent directement en local : ils clonent
`hello-groupe/helloworld-iac`, mettent à jour `kustomization.yaml` via PyYAML et poussent.

```bash
# Déployer la version "local" sur dev (utilise CI_COMMIT_SHORT_SHA=local)
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  --volume ../ci-templates:/ci-scripts:ro \
  deploy-dev

# Déployer un tag de release sur rec
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  --volume ../ci-templates:/ci-scripts:ro \
  --variable CI_COMMIT_TAG=v1.2.3 deploy-rec

# Rollback prod (annule un commit sur la branche main d'helloworld-iac)
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  --volume ../ci-templates:/ci-scripts:ro \
  --variable REVERT_SHA=abc1234 rollback-prod
```

### Jobs de build (to-be-continuous/docker)

`docker-buildah-build` et `docker-publish` viennent du component
[to-be-continuous/docker](https://gitlab.com/to-be-continuous/docker), inclus
par `ci-templates/templates/build-docker`. Buildah construit les images
**sans démon Docker** et les pousse directement sur GHCR
(`ghcr.io/k8s-gitops-lab`).

En local, ce job a besoin d'un `GHCR_TOKEN` valide (voir
`.gitlab-ci-local-secrets.yml`, gitignored) :

```bash
gitlab-ci-local --variables-file .gitlab-ci-local-secrets.yml \
  "docker-buildah-build: [helloworld-svc]"
```

`gitlab-ci-local` récupère aussi le component externe
`gitlab.com/to-be-continuous/docker` (pas seulement `shared-ci/ci-templates`
en interne) : par défaut via `git archive` en SSH, qui nécessite une clé SSH
enregistrée sur un compte gitlab.com même pour ce projet public.

> **Note ARM64** : Buildah tourne dans une image amd64. Sur Apple Silicon,
> elle s'exécute via la couche d'émulation Rosetta de Docker Desktop. Pour
> des builds locaux rapides, préférer `docker compose build`.

---

## Pipeline CI/CD complet

La chaîne de promotion est déclenchée depuis GitLab (push sur `main` ou tag `vX.Y.Z`) :

```
push main
  └─ docker-buildah-build  ──►  deploy-dev   (automatique)
     tag vX.Y.Z
  └─ docker-publish  ──►  deploy-rec      (automatique)
                     ──►  deploy-preprod  (manuel)
                     ──►  deploy-prod     (manuel)
```

Le tag est créé par le job `semantic-release`, exécuté automatiquement sur
`main` après `deploy-dev` : il analyse les Conventional Commits depuis le
dernier tag et pousse un tag `vX.Y.Z` (rien n'est publié si aucun commit
`feat:`/`fix:`/... n'est présent).
