# PRD

## Intention du projet

`helloworld` est l'application de référence du POC `poc-devops`. Elle démontre
le contrat applicatif attendu par la plateforme : un dépôt de code unique,
plusieurs services, une CI incluse depuis `infra/ci-templates` et un déploiement
GitOps vers le dépôt frère `helloworld-iac`.

La vision globale, le flow de release et les limites du POC sont décrits dans
`../../platform-cicd/docs/prd.md`.

## Produit attendu

L'application doit fournir un exemple simple mais réaliste d'onboarding :

- un backend HTTP exposant des routes de santé et de salutation ;
- un frontend statique capable d'appeler le backend ;
- un mode local via Docker Compose ;
- un pipeline CI/CD compatible avec le modèle trunk-based de la plateforme ;
- un monorepo multi-services utilisable comme modèle pour les futures apps.

## Utilisateurs cibles

- Développeur applicatif qui veut comprendre le format attendu par la
  plateforme.
- Mainteneur plateforme qui vérifie le comportement de bout en bout :
  build, push GHCR, mise à jour des manifests, synchronisation ArgoCD.
- Auteur d'une nouvelle app qui veut copier une structure minimale.

## Critères d'acceptation

- `docker compose up --build` démarre le frontend et le backend localement.
- Le backend répond sur `/`, `/hello/{name}` et `/health`.
- Le frontend est servi par nginx et appelle l'API via le routage applicatif.
- Le pipeline applicatif inclut le template CI versionné.
- Les services déclarés dans `SERVICES` ont chacun un sous-dossier et un
  `Dockerfile`.
- Les releases suivent le modèle plateforme : `semantic-release`, tag
  `vX.Y.Z`, promotion `rec` puis `preprod` et `prod`.

## Non-objectifs

- Fournir une application métier complexe.
- Tester tous les cas de sécurité applicative.
- Porter l'application hors du cluster local du POC sans adaptation.
