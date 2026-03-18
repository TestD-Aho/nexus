# PRD Audit — Refactoring de la Gestion des PDF

> **Projet** : Nexus CMS  
> **Module** : Gestion des Documents PDF  
> **Date** : 18 Mars 2026  
> **Priorité** : 🔴 Haute  

---

## 1. Audit de l'État Actuel

### 1.1 Constats

| Aspect | État Actuel | Problème Identifié |
|--------|-------------|-------------------|
| **Génération PDF (Typst)** | Documenté dans ARCHITECTURE.md §6 | ❌ **Non implémenté** — aucune dépendance Typst dans Cargo.toml, aucun service de génération |
| **Upload PDF** | Accepté via `accept="image/*,video/*,.pdf"` dans MediaUploader.jsx | ⚠️ Traitement générique — même pipeline qu'une image |
| **Modèle Media** | `media` table avec `mime_type`, `size`, `url` | ❌ Pas de métadonnées spécifiques PDF (pages, titre, auteur, version) |
| **Prévisualisation** | Gallery affiche `📄` pour tout non-image | ❌ Pas de rendu/preview PDF |
| **API Media** | `/media/upload` générique | ❌ Pas d'endpoint dédié PDF (génération CV, export, conversion) |
| **Storage** | Fichiers disques (`./uploads/{uuid}.{ext}`) | ⚠️ Pas de stratégie de versioning ni de CDN |
| **Sécurité** | Upload brut, pas de scan | ❌ Risque : fichiers malveillants déguisés en PDF |
| **OCR / Extraction** | Inexistant | ❌ Impossible d'indexer le contenu des PDF |
| **Export CV/Resume** | JSON Resume structuré (§5 ARCHITECTURE) | ❌ Pas de pipeline JSON→Typst→PDF |

### 1.2 Architecture Actuelle du Flux Média

```
Frontend (MediaUploader)
    │
    │ FormData + multipart
    ▼
POST /api/v1/media/upload
    │
    │ mime_guess::from_path(filename)
    │ tokio::fs::write(file)
    ▼
Table `media` (PostgreSQL)
    │
    │ URL = /uploads/{uuid}.{ext}
    ▼
ServeDir (fichiers statiques)
```

**Problème** : Le flux ne fait AUCUNE distinction entre PDF, image, ou vidéo. Un PDF est traité exactement comme un JPEG — zéro traitement, zéro validation, zéro enrichissement.

---

## 2. Objectifs du Refactoring

### 2.1 Vision

Transforme la gestion PDF de Nexus d'un simple "upload générique" vers un **système complet de gestion documentaire** avec génération, prévisualisation, versioning et extraction de contenu.

### 2.2 Objectifs SMART

| ID | Objectif | Métrique | Deadline |
|----|----------|----------|----------|
| O1 | Génération PDF depuis JSON Resume via Typst | Endpoint fonctionnel, PDF valide généré | Sprint 1 |
| O2 | Prévisualisation PDF dans la Media Gallery | Thumbnail + viewer intégré | Sprint 2 |
| O3 | Validation et sécurité des uploads PDF | Scan MIME réel, validation structure | Sprint 1 |
| O4 | Métadonnées PDF enrichies | Extraction titre/auteur/pages/nb de pages | Sprint 2 |
| O5 | API dédiée documents PDF | CRUD documents avec métadonnées | Sprint 2 |
| O6 | Versioning des documents | Historique des versions + diff | Sprint 3 |
| O7 | Extraction de texte (indexation) | Texte searchable par full-text search | Sprint 3 |

---

## 3. Architecture Cible

### 3.1 Vue d'Ensemble

```
┌─────────────────────────────────────────────────────────────────┐
│                      Frontend (React)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │ PDFUploader  │  │ PDFViewer    │  │ CVGenerator           │ │
│  │ (validation) │  │ (preview)    │  │ (JSON Resume → Typst) │ │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬───────────┘ │
└─────────┼─────────────────┼───────────────────────┼─────────────┘
          │                 │                       │
          ▼                 ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Backend (Rust/Axum)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │ PDF Service  │  │ Document     │  │ Typst Service         │ │
│  │ (validation, │  │ Repository   │  │ (generation,          │ │
│  │  extraction) │  │ (CRUD+ver.)  │  │  compilation)         │ │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬───────────┘ │
│         │                 │                       │             │
│         └─────────────────┼───────────────────────┘             │
│                           │                                     │
│                    PostgreSQL + Storage                          │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Nouveaux Modules Backend

```
backend/src/
├── services/
│   ├── mod.rs
│   ├── config.rs          # (+) pdf config
│   ├── app_state.rs
│   ├── auth.rs
│   ├── pdf.rs             # ← NOUVEAU : Service PDF
│   ├── typst.rs           # ← NOUVEAU : Moteur Typst
│   └── document.rs        # ← NOUVEAU : Gestion documents
├── models/
│   ├── mod.rs             # (+) Document, PdfMetadata, DocumentVersion
├── api/
│   ├── mod.rs             # (+) documents, pdf_generation
│   ├── documents.rs       # ← NOUVEAU : API Documents PDF
│   └── pdf_generation.rs  # ← NOUVEAU : API Génération
└── utils/
    └── pdf_validator.rs   # ← NOUVEAU : Validation PDF
```

### 3.3 Nouveaux Modèles de Données

#### Table `documents` (extension du système média)

```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id UUID REFERENCES media(id) ON DELETE CASCADE,
    title VARCHAR(500),
    author VARCHAR(255),
    page_count INTEGER,
    pdf_version VARCHAR(10),           -- ex: "1.7"
    is_encrypted BOOLEAN DEFAULT FALSE,
    has_text_content BOOLEAN DEFAULT FALSE,
    language VARCHAR(10),              -- ex: "fr", "en"
    document_type VARCHAR(50),         -- 'cv', 'resume', 'report', 'invoice', 'other'
    metadata JSONB DEFAULT '{}',       -- Métadonnées brutes XMP/PDF
    extracted_text TEXT,               -- Contenu textuel pour indexation
    checksum VARCHAR(64),              -- SHA-256 pour déduplication
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    media_id UUID REFERENCES media(id) ON DELETE CASCADE,
    changelog TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(document_id, version_number)
);

CREATE INDEX idx_documents_type ON documents(document_type);
CREATE INDEX idx_documents_text ON documents USING GIN(to_tsvector('french', extracted_text));
```

#### Modèles Rust

```rust
/// Document PDF enrichi
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub media_id: Uuid,
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: Option<i32>,
    pub pdf_version: Option<String>,
    pub is_encrypted: bool,
    pub has_text_content: bool,
    pub language: Option<String>,
    pub document_type: String,
    pub metadata: serde_json::Value,
    pub extracted_text: Option<String>,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Version d'un document
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_number: i32,
    pub media_id: Uuid,
    pub changelog: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Métadonnées extraites d'un PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
    pub page_count: u32,
    pub pdf_version: String,
    pub is_encrypted: bool,
    pub page_size: Option<PageSize>,
}

/// Taille de page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSize {
    pub width_pt: f64,
    pub height_pt: f64,
    pub name: Option<String>,  -- "A4", "Letter", etc.
}
```

---

## 4. Dépendances Techniques

### 4.1 Backend (Cargo.toml)

```toml
# PDF Processing
lopdf = "0.32"           # Lecture/écriture PDF bas niveau
pdf-extract = "0.7"      # Extraction de texte PDF
printpdf = "0.7"         # Génération PDF simple

# Typst (génération CV/Resume)
typst = "0.12"           # Moteur Typst
typst-render = "0.12"    # Rendu Typst → PDF

# Validation & Sécurité
sha2 = "0.10"            # SHA-256 checksums
magic = "0.13"           # Détection MIME réelle (pas seulement extension)

# Thumbnails
image = "0.25"           # Manipulation d'images (pour thumbnails)
poppler = "0.4"          # Rendu PDF → image (via poppler-rs)
```

### 4.2 Frontend

```json
{
  "react-pdf": "^7.7.0",        // Affichage PDF dans React
  "pdfjs-dist": "^4.0.0"        // PDF.js pour rendu client
}
```

---

## 5. Spécifications API

### 5.1 Endpoints Documents

| Méthode | Endpoint | Description | Auth |
|---------|----------|-------------|------|
| `POST` | `/api/v1/documents/upload` | Upload PDF avec validation enrichie | ✅ |
| `GET` | `/api/v1/documents` | Lister documents (filtres: type, langue, auteur) | ❌ |
| `GET` | `/api/v1/documents/:id` | Détail document + métadonnées | ❌ |
| `GET` | `/api/v1/documents/:id/preview` | Thumbnail/preview du PDF | ❌ |
| `GET` | `/api/v1/documents/:id/download` | Télécharger le PDF | ❌ |
| `GET` | `/api/v1/documents/:id/text` | Contenu textuel extrait | ❌ |
| `PUT` | `/api/v1/documents/:id` | Mettre à jour métadonnées | ✅ |
| `DELETE` | `/api/v1/documents/:id` | Supprimer document | ✅ |
| `POST` | `/api/v1/documents/:id/versions` | Uploader nouvelle version | ✅ |
| `GET` | `/api/v1/documents/:id/versions` | Historique des versions | ❌ |
| `GET` | `/api/v1/documents/search?q=` | Recherche full-text dans les PDF | ❌ |

### 5.2 Endpoints Génération

| Méthode | Endpoint | Description | Auth |
|---------|----------|-------------|------|
| `POST` | `/api/v1/pdf/cv` | Générer CV depuis JSON Resume | ✅ |
| `POST` | `/api/v1/pdf/cv/preview` | Prévisualiser CV (PNG/SVG) | ✅ |
| `GET` | `/api/v1/pdf/templates` | Lister templates Typst disponibles | ❌ |
| `POST` | `/api/v1/pdf/custom` | Générer PDF depuis template + data | ✅ |

### 5.3 Requêtes Types

#### Upload PDF Enrichi

```bash
curl -X POST http://localhost:3000/api/v1/documents/upload \
  -H "Authorization: Bearer <token>" \
  -F "file=@rapport.pdf" \
  -F "document_type=report" \
  -F "language=fr"
```

#### Génération CV

```bash
curl -X POST http://localhost:3000/api/v1/pdf/cv \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "template": "modern",
    "language": "fr",
    "data": {
      "basics": {
        "name": "Jean Dupont",
        "email": "jean@example.com",
        "location": { "city": "Paris" }
      },
      "work": [...],
      "education": [...],
      "skills": [...]
    }
  }'
```

#### Recherche Full-Text

```bash
curl "http://localhost:3000/api/v1/documents/search?q=machine+learning&lang=fr"
```

---

## 6. Flux de Génération PDF (Typst)

### 6.1 Pipeline Complet

```
JSON Resume (PostgreSQL)
    │
    ▼
┌─────────────────┐
│ Template Engine │  ← Sélection template (modern, classic, minimal)
│ (Typst Compiler)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Typst Source    │  ← Compilation .typ → .pdf
│ Compilation     │     typst::compile()
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ PDF Output      │  ← Stockage + métadonnées
│ Storage         │
└────────┬────────┘
         │
         ▼
    Response (binary)
```

### 6.2 Structure Templates Typst

```
backend/templates/pdf/
├── cv/
│   ├── modern.typ      # Design moderne avec couleurs
│   ├── classic.typ     # Design classique sobre
│   └── minimal.typ     # Design épuré minimal
├── invoice/
│   └── default.typ
├── report/
│   └── default.typ
└── shared/
    ├── colors.typ      # Palette de couleurs
    ├── fonts.typ       # Polices communes
    └── helpers.typ     # Fonctions utilitaires
```

---

## 7. Sécurité

### 7.1 Risques Identifiés

| Risque | Sévérité | Mitigation |
|--------|----------|------------|
| **Upload de PDF malveillant** (embedded JS, exploits) | 🔴 Critique | Validation MIME réelle (magic bytes), pas seulement extension |
| **PDF bomb** (zip bomb équivalent) | 🔴 Critique | Limite taille + pages max, scan avant stockage |
| **Extraction de données sensibles** | 🟡 Moyenne | Sanitization extracted_text, ACL sur endpoint text |
| **DDoS via génération** | 🟡 Moyenne | Rate limiting strict sur endpoints génération |
| **Storage non borné** | 🟡 Moyenne | Quotas utilisateur, nettoyage automatique anciens PDF |

### 7.2 Validation Upload PDF

```rust
pub struct PdfValidator {
    max_size: usize,           // 50MB par défaut
    max_pages: u32,            // 500 pages max
    allowed_versions: Vec<String>,  // ["1.4", "1.5", "1.6", "1.7", "2.0"]
}

impl PdfValidator {
    pub fn validate(&self, data: &[u8]) -> Result<PdfMetadata, ValidationError> {
        // 1. Vérifier magic bytes (%PDF-)
        self.check_magic_bytes(data)?;
        
        // 2. Vérifier taille
        self.check_size(data)?;
        
        // 3. Parser le PDF (lopdf)
        let metadata = self.extract_metadata(data)?;
        
        // 4. Vérifier le nombre de pages
        self.check_page_count(&metadata)?;
        
        // 5. Vérifier version PDF
        self.check_version(&metadata)?;
        
        // 6. Détecter contenu potentiellement dangereux
        self.check_dangerous_content(data)?;
        
        Ok(metadata)
    }
}
```

---

## 8. Plan de Migration

### 8.1 Phase 1 — Fondations (Sprint 1)

```
✅ Ajouter dépendances Cargo (lopdf, pdf-extract, sha2, image)
✅ Créer modèle Document + DocumentVersion + migration SQL
✅ Créer PdfValidator avec validation magic bytes
✅ Refactorer upload_media → détecter PDF et enrichir métadonnées
✅ Créer service pdf.rs (extraction texte, métadonnées, checksums)
✅ Endpoint POST /documents/upload avec validation enrichie
✅ Endpoint GET /documents avec filtres
✅ Tests unitaires validation PDF
```

### 8.2 Phase 2 — Visualisation (Sprint 2)

```
✅ Créer service typst.rs (génération PDF)
✅ Endpoint POST /pdf/cv (JSON Resume → Typst → PDF)
✅ Intégrer react-pdf dans frontend pour preview
✅ Endpoint GET /documents/:id/preview (thumbnail)
✅ Créer composant DocumentViewer dans frontend
✅ Créer composant CVGenerator dans frontend
✅ Endpoint GET /documents/search?q= (full-text)
✅ Templates Typst CV (modern, classic, minimal)
```

### 8.3 Phase 3 — Avancé (Sprint 3)

```
✅ Versioning documents (POST /documents/:id/versions)
✅ Historique versions (GET /documents/:id/versions)
✅ Déduction checksums (SHA-256) pour déduplication
✅ Cache de génération PDF (Redis)
✅ Export multiple formats (PDF/A, PDF/X)
✅ Batch processing (générer CV dans plusieurs langues)
✅ Webhooks sur génération terminée
✅ Dashboard analytics documents (stats admin)
```

---

## 9. Impact sur le Code Existant

### 9.1 Fichiers à Modifier

| Fichier | Modification |
|---------|-------------|
| `backend/Cargo.toml` | Ajouter dépendances PDF |
| `backend/src/models/mod.rs` | Ajouter `Document`, `DocumentVersion`, DTOs |
| `backend/src/api/media.rs` | Enrichir `upload_media()` pour détection PDF |
| `backend/src/api/mod.rs` | Ajouter routes `documents` et `pdf_generation` |
| `backend/src/services/config.rs` | Ajouter config PDF (max_size, templates_dir, etc.) |
| `backend/src/db/mod.rs` | Ajouter migration tables `documents` + `document_versions` |
| `backend/src/main.rs` | Initialiser services PDF/Typst au démarrage |
| `frontend/src/api/index.js` | Ajouter API client pour documents + PDF generation |
| `frontend/src/components/MediaUploader.jsx` | Ajouter validation PDF côté client |
| `docker-compose.yml` | Volume pour templates Typst |

### 9.2 Fichiers à Créer

| Fichier | Description |
|---------|-------------|
| `backend/src/services/pdf.rs` | Service principal PDF |
| `backend/src/services/typst.rs` | Moteur de génération Typst |
| `backend/src/api/documents.rs` | API Documents PDF |
| `backend/src/api/pdf_generation.rs` | API Génération PDF |
| `backend/src/utils/pdf_validator.rs` | Validation et sécurité PDF |
| `backend/templates/pdf/cv/modern.typ` | Template CV moderne |
| `backend/templates/pdf/cv/classic.typ` | Template CV classique |
| `backend/templates/pdf/cv/minimal.typ` | Template CV minimal |
| `frontend/src/components/PDFViewer.jsx` | Composant visualisation PDF |
| `frontend/src/components/DocumentUploader.jsx` | Upload PDF enrichi |
| `frontend/src/components/CVGenerator.jsx` | Interface génération CV |
| `frontend/src/components/DocumentCard.jsx` | Carte document dans gallery |
| `frontend/src/pages/DocumentsPage.jsx` | Page gestion documents |

---

## 10. Métriques de Succès

| Métrique | Cible | Mesure |
|----------|-------|--------|
| Temps upload PDF | < 2s (validation incluse) | Logs backend |
| Temps génération CV | < 5s | Logs backend |
| Précision extraction texte | > 95% | Tests sur corpus |
| Disponibilité API | > 99.9% | Monitoring |
| Taille PDF généré | < 500KB (CV standard) | Inspection fichiers |
| Pages PDF max supportées | 500 pages | Tests charge |
| XSS/Injection via PDF | 0 incidents | Pentest |

---

## 11. Risques & Dépendances

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| Typst API instable (0.x) | Réécriture service | Moyenne | Abstraction via trait, fallback printpdf |
| lopdf limité pour PDF complexes | Parsing échoue | Basse | Fallback pdf-extract pour extraction seule |
| Poppler pas disponible en Docker | Pas de thumbnails | Moyenne | ImageMagick comme fallback, ou rendu côte client |
| Volume PDF augmente storage | Coûts infrastructure | Haute | Déduplication par checksum, compression, CDN |

---

## 12. Glossaire

| Terme | Définition |
|-------|-----------|
| **Typst** | Langage de typographie moderne, alternative à LaTeX |
| **JSON Resume** | Standard ouvert pour structurer des CV en JSON |
| **lopdf** | Crate Rust pour lecture/écriture bas niveau PDF |
| **XMP** | Extensible Metadata Platform — standard de métadonnées PDF |
| **PDF/A** | Sous-standard PDF pour archivage long terme |
| **Magic bytes** | Premiers octets d'un fichier permettant d'identifier son type réel |

---

*Document généré le 18 Mars 2026 — Nexus CMS v1.0*
