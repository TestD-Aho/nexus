# Rapport d'Architecture - Nexus CMS

> **Projet** : Plateforme CMS Headless, Portfolio Modulaire et Moteur Dynamique de CV  
> **Backend** : Rust (Axum) + PostgreSQL  
> **Frontend** : React + Vite  
> **Date** : Mars 2026

---

## 1. Introduction

Nexus est une plateforme CMS Headless entièrement pilotée par API, conçue pour une scalabilité et sécurité de niveau entreprise. L'architecture repose sur le paradigme de modularité absolue où chaque élément d'interface opère comme un bloc indépendant.

### 1.1 Caractéristiques Clés

- 🔧 **Architecture Headless** - Séparation contenu/présentation
- 🧱 **Moteur de Blocs** - Composants réutilisables (Hero, RichText, ProjectGrid, etc.)
- 🔐 **Sécurité Zero-Trust** - RBAC + ABAC avec Casbin
- 📄 **Génération PDF** - Moteur Typst intégré
- 🌐 **i18n Native** - Support multilingue intégré

---

## 2. Modélisation des Données

### 2.1 Structure des Blocs

```json
{
  "id": "uuid-v4",
  "type": "HeroHeader|RichText|ProjectGrid|SkillMatrix|ContactForm|TestimonialSlider",
  "order": 0,
  "status": "draft|published|archived",
  "content": { },
  "styling": { },
  "access_control": { }
}
```

### 2.2 Collections Dynamiques

- **Expériences Professionnelles**
- **Articles de Blog**
- **Réalisations de Projets**

### 2.3 PostgreSQL + JSONB

| Type | Usage |
|------|-------|
| Colonnes typées | Données relationnelles (users, roles) |
| JSONB | Configurations de blocs, métadonnées |

---

## 3. Backend (Rust/Axum)

### 3.1 Stack Technique

- **Runtime** : Rust + Tokio
- **Framework** : Axum
- **ORM** : SQLx
- **Auth** : JWT (HS512) + Argon2
- **Permissions** : Casbin

### 3.2 API Endpoints

| Module | Routes |
|--------|--------|
| Auth | `/auth/login`, `/auth/register`, `/auth/me` |
| Pages | CRUD complet |
| Blocks | CRUD + reorder |
| Media | Upload, delete |
| Admin | Users, roles, stats |
| System | Feature flags, maintenance |

---

## 4. Sécurité

### 4.1 Rôles

| Rôle | Permissions |
|------|-------------|
| Super-Architecte | Accès root complet |
| Gestionnaire | CRUD contenu |
| VIP | Lecture publique |
| Visiteur | Lecture seule |

### 4.2 Authentification

- **Access Token** : 15 min
- **Refresh Token** : Cookie HttpOnly
- **Middleware** : Filtrage par route (GET public, POST/PUT/DELETE protégé)

---

## 5. Intégration JSON Resume

### 5.1 Structure

```json
{
  "basics": { "name", "email", "location" },
  "work": [ ],
  "education": [ ],
  "skills": [ ]
}
```

### 5.2 i18n

Stockage multilingue dans JSONB :
```json
{
  "fr": { ... },
  "en": { ... }
}
```

---

## 6. Génération PDF (Typst)

### Processus

1. Extraction JSON depuis PostgreSQL
2. Assembly markup Typst
3. Compilation native → PDF
4. Retour response binaire

---

## 7. Performances

### Benchmarks Attendus

- **Latence API** : < 10ms (p50)
- **Mémoire** : < 50MB idle
- **Concurrent connections** : 10,000+

---

## 8. Déploiement

### Docker Compose

```yaml
services:
  postgres:
    image: postgres:15-alpine
  backend:
    build: ./backend
  frontend:
    build: ./frontend
```

---

## 9. Perspectives

### Améliorations V2

1. **SSR/SSG** : Next.js ou Astro pour SEO
2. **Webhooks** : Déclencheurs sur publication
3. **WYSIWYG** : TipTap ou Quill
4. **Versioning** : Historique des blocs

---

*Document généré pour Nexus CMS v1.0*
