# Spec fonctionnelle

## Parcours local

Le développeur peut lancer l'application avec `docker compose up --build`.
Deux services sont démarrés :

- `helloworld-gui`, disponible sur `http://localhost:8080` ;
- `helloworld-svc`, disponible sur `http://localhost:8081`.

Le frontend présente une interface de démonstration et interroge le backend.
Le backend retourne des réponses JSON simples.

## API backend

Le service `helloworld-svc` expose :

- `GET /` : retourne le message par défaut `Hello, World!` ;
- `GET /hello/{name}` : retourne une salutation personnalisée ;
- `GET /health` : retourne l'état de santé du service.

Le paramètre `name` est nettoyé côté serveur. Une valeur vide retombe sur
`World`; une valeur trop longue est refusée avec une erreur HTTP 400.

## Comportement CI/CD

Le dépôt suit le flow défini par `platform-cicd` :

- `main` est l'unique branche longue durée du code ;
- chaque merge sur `main` déclenche un build mutable et un déploiement dev ;
- une release est créée par le job manuel `semantic-release` ;
- le tag `vX.Y.Z` déclenche le pipeline de promotion ;
- les promotions modifient le dépôt `helloworld-iac`, pas le cluster
  directement.

## Monorepo multi-services

Le versioning est commun à l'application. Une release reconstruit les deux
services sous le même tag :

- `helloworld-svc` ;
- `helloworld-gui`.

Il n'y a pas de version indépendante par service dans ce POC.

## Dépendances fonctionnelles

`helloworld` dépend de :

- `ci-templates` pour la logique CI partagée ;
- `helloworld-iac` pour l'état GitOps des environnements ;
- `platform-cicd` pour GitLab, ArgoCD et le runner (les images sont poussées
  sur GHCR, pas sur un registry géré par `platform-cicd`).
