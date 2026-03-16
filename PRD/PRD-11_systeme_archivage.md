# PRD — Système d'Archivage et d'Historisation CheckMaster

**Version**: 1.0 
**Date**: 15 mars 2026 
**Auteur**: Sisyphus 
**Statut**: Brouillon 
**Contrainte critique**: Aucune modification de base de données — utiliser les tables existantes uniquement.

---

## 1. Contexte et Objectifs

### 1.1 Contexte

CheckMaster est un système de gestion des soutances universitaires qui orchestre le parcours complet d'un étudiant : inscription → dépôt de rapport → validation par commission → programmation de soutenance → évaluation → délibération finale.

À ce jour, le système dispose de briques partielles d'archivage :
- Un modèle `Archive.php` (1029 lignes) qui reconstruit l'historique à partir des tables métier existantes
- Un modèle `AuditLog.php` qui journalise dans la table `pister`
- Des contrôleurs d'archives pour les documents (PDF, ZIP)
- Un stockage filesystem pour les rapports et comptes rendus

**Ce qui manque** :
- Traçabilité complète du cycle de vie (transitions partiellement journalisées)
- Versionnement des documents (stockage plat, pas de versions)
- Reporting historique et analytique complet
- Capacités d'export conformité/légal
- Recherche transversale dans le temps
- Automatisation des transitions d'année académique
- Politiques de rétention (seulement nettoyage audit log à 30 jours)
- Pas de champ détails/JSON dans `pister` pour un journal enrichi

### 1.2 Objectifs

| # | Objectif | Mesure de succès |
|---|----------|------------------|
| O1 | Archiver automatiquement chaque dossier étudiant à la clôture d'année | 100% des dossiers archivés sans intervention manuelle |
| O2 | Fournir une traçabilité complète des actions système | Toute action utilisateur critique journalisée dans `pister` |
| O3 | Permettre la consultation historique multi-années | Accès à tout dossier depuis n'importe quelle année académique |
| O4 | Générer des exports conformité/légal complets | Export d'un dossier étudiant en < 30 secondes |
| O5 | Assurer l'intégrité des documents archivés | Vérification par checksum SHA-256 |
| O6 | Faciliter la recherche historique | Recherche par étudiant, année, thème, entreprise, jury |

### 1.3 Portée

#### In scope
- Archivage logique des dossiers étudiants (lecture depuis tables existantes)
- Organisation filesystem des documents par année/étudiant
- Journalisation d'audit renforcée (via table `pister` existante)
- Exports conformité (dossier complet, année académique, audit)
- Recherche et consultation historique
- Transitions d'année académique automatisées
- Reporting analytique multi-années
- Vérification d'intégrité des documents archivés

#### Out scope
- Création de nouvelles tables ou modification de schéma
- Migration de données existantes (utilisation in-place)
- Archivage physique (suppression des données opérationnelles)
- Intégration avec des systèmes externes d'archivage légal
- Modification des workflows existants d'inscription/évaluation

---

## 2. Utilisateurs et Cas d'Usage

### 2.1 Personas

| Persona | Rôle | Besoin archivage |
|---------|------|------------------|
| **Secrétaire académique** | Administration quotidienne | Consulter historique d'un étudiant, générer attestation |
| **Chef de département** | Supervision filière | Reporting annuel, statistiques multi-années |
| **Membre de commission** | Validation rapports | Historique des décisions, comparaison inter-années |
| **Enseignant jury** | Évaluation soutenances | Historique des évaluations passées |
| **Responsable qualité** | Conformité | Export légal, audit trail, vérification intégrité |

### 2.2 Cas d'Usage Principaux

#### CU-1 : Consulter l'historique complet d'un étudiant
```
En tant que secrétaire académique,
Je veux consulter le parcours complet d'un étudiant (toutes années confondues),
Afin de répondre à une demande d'attestation ou de vérification.

Parcours :
1. Recherche étudiant par nom/numéro carte
2. Affichage timeline : inscription → rapport → dépôt → validation → candidature → soutenance → évaluation → note finale → compte rendu
3. Accès aux documents (rapports, CR, PV) par version
4. Affichage des décisions et commentaires associés
```

#### CU-2 : Générer un export conformité pour un étudiant
```
En tant que responsable qualité,
Je veux générer un ZIP contenant l'intégralité du dossier d'un étudiant,
Afin de répondre à une demande légale ou administrative.

Contenu du ZIP :
- Fiche d'identité académique
- Historique des inscriptions
- Toutes les versions des rapports
- Historique des validations/rejets avec commentaires
- Candidature et programmation de soutenance
- Composition du jury
- Évaluations détaillées par critère
- Notes finales
- Compte rendu de délibération
- Extrait d'audit lié au dossier
- Manifeste avec checksums SHA-256
```

#### CU-3 : Archiver un étudiant lors de la clôture d'année
```
En tant que système (automatisé),
Je veux archiver le dossier complet des étudiants d'une année académique,
Afin de préserver l'historique et libérer la vue opérationnelle.

Parcours :
1. Identifier tous les étudiants inscrits pour l'année N
2. Pour chaque étudiant, construire le dossier complet
3. Copier les documents dans ressources/uploads/archives/YYYY-YYYY/<num_etu>/
4. Générer les fichiers métadonnées JSON (sidecars)
5. Calculer les checksums SHA-256
6. Générer le manifeste de l'année
7. Journaliser l'opération dans pister
```

#### CU-4 : Rechercher dans les archives
```
En tant que chef de département,
Je veux rechercher dans les archives par critères multiples,
Afin d'analyser les tendances ou retrouver un dossier spécifique.

Critères de recherche :
- Étudiant (nom, numéro carte)
- Année académique
- Thème de rapport (mot-clé)
- Entreprise d'accueil
- Membre du jury
- Statut (validé, rejeté, soutenu)
- Mention obtenue
```

#### CU-5 : Générer un reporting analytique multi-années
```
En tant que chef de département,
Je veux visualiser des statistiques sur plusieurs années académiques,
Afin d'identifier des tendances et piloter la qualité.

Indicateurs :
- Taux de réussite par année
- Distribution des mentions
- Top entreprises d'accueil
- Évolution de la moyenne générale
- Participation des enseignants aux jurys
- Taux de validation des rapports
```

---

## 3. Architecture Fonctionnelle

### 3.1 Principe Fondamental

**Architecture d'archive logique, pas physique.**

L'historique est reconstruit à partir :
- Des tables métier existantes (source de vérité)
- De la table `pister` pour l'audit
- Du filesystem pour les documents archivés
- De requêtes SQL de reconstruction temporelle

Aucune table d'archive dédiée n'est créée.

### 3.2 Modèle de Cycle de Vie

Le parcours d'un étudiant est modélisé comme une machine à états applicative (13 étapes) :

| Étape | Source | Champ date clé | États possibles | Transition |
|-------|--------|----------------|-----------------|------------|
| 1. Inscription | `inscriptions` | `date_inscription` | Active, Soldée | → Stage ou Rapport |
| 2. Stage | `informations_stage` | `date_debut_stage` | En cours, Terminé | → Rapport |
| 3. Rédaction rapport | `rapport_etudiants` | `date_redaction_rapport` | `en_cours` | → Dépôt |
| 4. Dépôt rapport | `deposer` | `date_depot` | Soumis | → Validation |
| 5. Validation | `valider` + `rapport_etudiants.statut_rapport` | `date_validation` | Validé, Rejeté | Validé → Candidature ; Rejeté → Rédaction |
| 6. Candidature | `candidature_soutenance` | `date_candidature` | `En attente`, `Validée`, `Rejetée` | Validée → Programmation |
| 7. Programmation | `programmer_soutenance` | `date_soutenance` | Programmée | → Composition jury |
| 8. Composition jury | `enseignant_jury` | `date_composer_jury` | Complet, Incomplet | Complet → Soutenance tenue |
| **9. Soutenance tenue** | `programmer_soutenance.date_soutenance` + `enseignant_jury` | `date_soutenance` | Tenue, Reportée, Absent | → Évaluation |
| **10. Délibération** | `compte_rendu` | `date_CR` | En délibération | → Mention |
| **11. Attribution mention** | `notes` | `date_creation` | Mention attribuée | → Compte rendu final |
| 12. Compte rendu final | `compte_rendu` | `date_CR` | Publié | → Clôture |
| 13. Clôture dossier | Agrégat | `date_modification` (notes) | Archivé | Fin du cycle |

**Note sur la distinction soutenance/délibération/mention** : L'étape 9 (soutenance tenue) est l'événement physique. L'étape 10 (délibération) correspond au moment où le jury délibère et produit le compte rendu. L'étape 11 (mention) est l'enregistrement officiel de la note/mention dans le système. Ces trois étapes sont distinctes fonctionnellement même si elles peuvent se succéder rapidement.

### 3.3 Tables Exploitées

Voir l'**Annexe A** pour le mapping complet des 62 tables explorées. Ci-dessous, les tables directement exploitées par le système d'archivage.

#### Tables d'ancrage identitaire
| Table | Rôle archivage | Colonnes clés |
|-------|----------------|---------------|
| `etudiants` | Identité racine | `num_carte_etud`, `nom_etu`, `prenom_etu`, `promotion_etu` |
| `enseignants` | Identité jury/encadrant | `id_enseignant`, `nom_enseignant`, `prenom_enseignant` |
| `utilisateur` | Traçabilité acteur audit | `id_utilisateur`, `login`, `nom`, `prenom`, `actif` |
| `annee_academique` | Bornage temporel | `id_annee_acad`, `date_deb`, `date_fin` |
| `entreprises` | Dimension employeur | `id_entreprise`, `lib_long_entreprise` |
| `maitre_de_stage` | Maître de stage | `id_maitre_stage`, nom, prénom |
| `personnel_admin` | Personnel administratif | `id_pers_admin`, nom, prénom |
| `domaine` | Domaine de spécialité | `id_domaine`, `libelle` |
| `specialite` | Spécialité | `id_specialite`, `libelle` |
| `salles` | Salles de soutenance | `id_salle`, `nom_salle`, `capacite` |

#### Tables de cycle de vie
| Table | Rôle archivage | Colonnes clés |
|-------|----------------|---------------|
| `inscriptions` | Historique inscription | `num_carte_etud`, `id_annee_acad`, `date_inscription`, `num_versement` |
| `informations_stage` | Contexte stage | `num_etu`, `id_entreprise`, `date_debut_stage`, `date_fin_stage`, `sujet_stage` |
| `rapport_etudiants` | Versions rapport | `id_rapport`, `num_etu`, `version`, `statut_rapport`, `date_redaction_rapport` |
| `deposer` | Événement dépôt | `num_etu`, `id_rapport`, `date_depot` |
| `valider` | Décision validation | `id_rapport`, `date_validation`, `decision_validation`, `commentaire_validation` |
| `affecter` | Affectation encadrant | `id_enseignant`, `role`, `id_rapport`, `id_jury` |
| `candidature_soutenance` | Candidature soutenance | `num_etu`, `date_candidature`, `statut_candidature`, `commentaire_admin` |
| `programmer_soutenance` | Événement soutenance | `num_etud`, `date_soutenance`, `heure_soutenance`, `id_annee_acad`, `theme_soutenance` |
| `enseignant_jury` | Composition jury | `num_soutenance`, `id_enseignant`, `id_qualite_jury`, `date_composer_jury` |
| `qualite_jury` | Rôles jury | `id_qualite_jury`, `lib_qualite` (DM, EN, EX, MS, PJ) |
| `evaluer` | Scores détaillés | `num_etudiant`, `id_critere`, `note`, `date_eval` |
| `critere_evaluation` | Critères d'évaluation | `id_critere`, `lib_critere`, `bareme` |
| `bareme_critere` | Barème par critère | `id_critere`, `id_annee_acad`, `bareme_max` |
| `notes` | Résultat final | `num_etu`, `id_annee_acad`, `moyenne_M1`, `moyenne_M2`, `date_creation` |
| `compte_rendu` | Compte rendu délibération | `id_CR`, `num_etu`, `contenu_CR`, `chemin_fichier_pdf`, `date_CR` |
| `compte_rendu_rapport` | Lien CR-rapport | `id_CR`, `id_rapport` |
| `rendre` | Transmission CR | `id_CR`, `id_enseignant`, `date_transmission` |

#### Tables de workflow (statuts/validations)
| Table | Rôle archivage | Colonnes clés |
|-------|----------------|---------------|
| `session` | Session académique | `id_session`, `lib_session`, `date_deb`, `date_fin` |
| `candidature_soutenance` | Transition rapport → soutenance | `statut_candidature` |

#### Table d'audit
| Table | Rôle archivage | Colonnes clés |
|-------|----------------|---------------|
| `pister` | Journal d'audit système | `id_piste`, `id_utilisateur`, `action`, `nom_table`, `statut_action`, `date_creation` |

#### Tables hors périmètre archivage (non exploitées)
| Table | Raison exclusion |
|-------|------------------|
| `reclamations` | Gestion opérationnelle, pas d'archive |
| `messages` | Communication interne |
| `fonctionnalites`, `permissions`, `type_utilisateur`, `groupe_utilisateur` | Configuration système/rôles |

### 3.4 Organisation Filesystem

Les documents archivés sont organisés dans une structure canonique par année académique et par étudiant. Chaque document archivé est accompagné d'un fichier de métadonnées (sidecar JSON) contenant les checksums SHA-256 pour vérification d'intégrité.

**Règles de nommage** :
- Dossier année : `YYYY-YYYY` (ex: `2025-2026`)
- Dossier étudiant : `<num_carte_etud>`
- Documents : nom descriptif avec version (`rapport_<id>_v<version>.pdf`)
- Métadonnées : `<nom_document>_metadata.json`
- Manifeste : `manifeste_annee.json` (par année), `manifeste_etudiant.json` (par étudiant)

**Structure détaillée et exemples** : Voir **Annexe B**.

---

## 4. Spécifications Fonctionnelles

### 4.1 Module Historique Étudiant

#### 4.1.1 Vue Timeline
- Afficher le parcours complet d'un étudiant sous forme de timeline chronologique
- Chaque événement = une étape du cycle de vie avec date, description, acteur
- Groupement par année académique
- Accès direct aux documents associés (rapport, CR, PV)
- Affichage des décisions (validation, rejet) avec commentaires

#### 4.1.2 Vue Dossier Complet
- Informations personnelles (depuis `etudiants`)
- Historique des inscriptions (depuis `inscriptions`, groupé par année)
- Informations stage (depuis `informations_stage` + `entreprises`)
- Rapports avec versions (depuis `rapport_etudiants`)
- Validation/rejet (depuis `valider`)
- Candidature soutenance (depuis `candidature_soutenance`)
- Programmation et jury (depuis `programmer_soutenance` + `enseignant_jury`)
- Évaluations détaillées (depuis `evaluer` + `critere_evaluation`)
- Notes finales (depuis `notes`)
- Compte rendu (depuis `compte_rendu`)
- Journal d'audit lié (depuis `pister`, corrélé par date/acteur)

### 4.2 Module Audit Renforcé

#### 4.2.1 Vocabulaire d'Actions Standardisé
Utiliser des valeurs `action` contrôlées dans `pister` :

| Action | Quand journaliser | `nom_table` |
|--------|-------------------|-------------|
| `Création` | Nouvel enregistrement | Table concernée |
| `Modification` | Mise à jour existante | Table concernée |
| `Suppression` | Suppression logique/physique | Table concernée |
| `Connexion` | Login utilisateur | `session` |
| `Déconnexion` | Logout utilisateur | `session` |
| `Dépôt` | Dépôt de rapport | `deposer` |
| `Validation` | Validation de rapport | `valider` |
| `Rejet` | Rejet de rapport | `valider` |
| `Evaluation` | Saisie d'évaluation | `evaluer` |
| `Exportation` | Export de données | `exports_conformite` |
| `Impression` | Impression document | Table concernée |
| `Archivage` | Archivage dossier/année | `archives_documents` |
| `Consultation archive` | Lecture archive | `archives_documents` |
| `Clôture année` | Transition année | `annee_academique` |

#### 4.2.2 Limitations Acceptées
La table `pister` existante n'a pas de colonne pour l'ID de l'objet concerné ni pour les valeurs avant/après. Compensations :
- Corrélation par `id_utilisateur` + `nom_table` + `action` + `date_creation` + fenêtre temporelle
- Pour les exports, inclure l'étudiant/année dans le nom de fichier généré
- Joiner systématiquement avec `utilisateur` pour l'affichage de l'acteur

### 4.3 Module d'Export Conformité

#### 4.3.1 Export Dossier Étudiant
Générer un ZIP contenant :
- [ ] Fiche d'identité académique (PDF généré)
- [ ] Historique des inscriptions (CSV)
- [ ] Informations stage (CSV)
- [ ] Toutes les versions des rapports (PDF/HTML originaux)
- [ ] Historique des dépôts (CSV)
- [ ] Décisions de validation avec commentaires (CSV)
- [ ] Candidature et programmation (CSV)
- [ ] Composition du jury (CSV)
- [ ] Évaluations détaillées par critère (CSV)
- [ ] Notes finales (CSV)
- [ ] Compte rendu de délibération (PDF)
- [ ] Extrait d'audit lié au dossier (CSV)
- [ ] Manifeste avec checksums SHA-256 (JSON)

**Critère de performance** : Génération en < 30 secondes pour un dossier standard.

#### 4.3.2 Export Année Académique
Générer un ZIP contenant :
- [ ] Liste des étudiants inscrits (CSV)
- [ ] Rapports et statuts (CSV)
- [ ] Candidatures (CSV)
- [ ] Soutenances programmées (CSV)
- [ ] Compositions des jurys (CSV)
- [ ] Évaluations (CSV)
- [ ] Notes finales (CSV)
- [ ] Extrait d'audit de l'année (CSV)
- [ ] Index des documents avec checksums (JSON)

#### 4.3.3 Export Audit
- [ ] Par utilisateur (filtre date)
- [ ] Par plage de dates
- [ ] Par table/module
- [ ] Par année académique

#### 4.3.4 Journalisation des Exports
Tout export est journalisé dans `pister` :
- `action = 'Exportation'`
- `nom_table = 'exports_conformite'`
- `statut_action = 'Succès'` ou `'Erreur'`

### 4.4 Module de Recherche Historique

#### 4.4.1 Critères de Recherche
- Étudiant : nom, prénom, numéro de carte
- Année académique : liste déroulante
- Thème de rapport : recherche plein texte
- Entreprise d'accueil : liste déroulante + recherche
- Membre du jury : liste déroulante
- Statut : Validé, Rejeté, Soutenu, En cours
- Mention : liste déroulante
- Plage de dates : date début / date fin

#### 4.4.2 Résultats
- Liste paginée avec tri par date (décroissant par défaut)
- Aperçu rapide (nom, année, statut, thème)
- Accès direct au dossier complet
- Export des résultats de recherche (CSV)

### 4.5 Module de Reporting Analytique

#### 4.5.1 Indicateurs Multi-années
- Taux de réussite par année académique
- Distribution des mentions (Très Bien, Bien, Assez Bien, Passable)
- Top 10 des entreprises d'accueil
- Évolution de la moyenne générale (M1, M2)
- Nombre de soutenances par année
- Taux de validation des rapports
- Participation des enseignants aux juries (nombre de jurys/enseignant/an)

#### 4.5.2 Visualisations
- Graphiques linéaires pour les tendances multi-années
- Diagrammes circulaires pour les distributions
- Tableaux comparatifs année sur année
- Export des graphiques (PNG) et données (CSV)

### 4.6 Module de Transition d'Année Académique

#### 4.6.1 Processus de Clôture
Exécuté automatiquement ou manuellement lors du passage à une nouvelle année :

1. **Geler le périmètre** : Identifier tous les étudiants liés à `id_annee_acad`
2. **Construire les dossiers** : Pour chaque étudiant, assembler le dossier complet
3. **Copier les documents** : Vers `ressources/uploads/archives/YYYY-YYYY/<num_etu>/`
4. **Générer les métadonnées** : Fichiers sidecar JSON + checksums SHA-256
5. **Générer le manifeste** : Manifeste de l'année avec statistiques
6. **Journaliser** : `pister.action = 'Clôture année'`, `nom_table = 'annee_academique'`
7. **Préserver la continuité** : Ne pas déplacer les lignes BD, ne pas modifier les FK

#### 4.6.2 Préservation Opérationnelle
- Les données restent dans les tables métier (pas de migration)
- Les liens opérationnels restent intacts
- L'historique reste consultable via `id_annee_acad`
- Les étudiants actifs sont reportés via de nouvelles inscriptions

### 4.7 Module d'Intégrité des Documents

#### 4.7.1 Vérification Checksum
- Calcul SHA-256 à l'archivage
- Stockage dans les sidecars JSON
- Vérification à la demande (manuelle ou planifiée)
- Rapport de réconciliation des fichiers manquants

#### 4.7.2 Politique de Rétention
| Type de document | Rétention | Justification |
|------------------|-----------|---------------|
| Rapports validés | Permanente | Valeur académique |
| Rapports rejetés | 5 ans | Référence intermédiaire |
| Comptes rendus | Permanente | Valeur légale |
| PV finaux | Permanente | Valeur légale |
| Fiches inscription | 10 ans | Conformité administrative |
| Journal d'audit | 5 ans (nettoyage existant à 30j → étendre) | Traçabilité |
| Exports conformité | 7 ans | Obligation légale |

---

## 5. Contraintes Techniques

### 5.1 Contrainte Absolue
**Aucune modification de base de données.** Toutes les fonctionnalités doivent être implémentées en utilisant les tables et colonnes existantes.

### 5.2 Technologies Disponibles
- PHP MVC custom (pas Laravel/Symfony)
- MySQL via PDO
- TCPDF pour la génération PDF
- JavaScript vanilla (frontend)
- Stockage filesystem pour les documents

### 5.3 Composants Existants à Réutiliser
Le système dispose déjà de briques d'archivage qui doivent être étendues, pas réécrites :
- **Modèle de lecture historique** : reconstruction de l'historique depuis les tables métier (existant, ~1000 lignes)
- **Modèle d'audit** : journalisation dans `pister` (existant, méthodes de log par type d'action)
- **Service d'orchestration** : coordination données d'archive avec filtrage par année/statut/recherche
- **Contrôleurs d'archive** : dossier étudiant, documents (PDF/ZIP), soutenances, comptes rendus
- **Service de génération PDF** : génération de documents PDF (TCPDF)

### 5.4 Contraintes de Performance
- Recherche historique : < 3 secondes pour 1000 résultats
- Chargement timeline étudiant : < 2 secondes
- Export dossier complet : < 30 secondes
- Export année académique : < 2 minutes
- Vérification checksum : < 5 minutes pour 500 documents

---

## 6. Critères d'Acceptation

### 6.1 Historique Étudiant
- [ ] CA-1 : Depuis la vue archives, je peux rechercher un étudiant par nom ou numéro de carte et obtenir des résultats en < 3 secondes
- [ ] CA-2 : En cliquant sur un étudiant, je vois sa timeline complète avec tous les événements de son parcours, triés chronologiquement
- [ ] CA-3 : Chaque événement de la timeline affiche la date, la description et l'acteur responsable
- [ ] CA-4 : Je peux accéder aux documents (rapports, CR, PV) directement depuis la timeline
- [ ] CA-5 : Je vois les commentaires de validation/rejet associés aux rapports

### 6.2 Audit
- [ ] CA-6 : Les actions suivantes sont systématiquement journalisées dans `pister` : Création, Modification, Suppression, Connexion, Déconnexion, Dépôt, Validation, Rejet, Évaluation, Exportation, Impression, Archivage, Consultation archive, Clôture année
- [ ] CA-7 : Les entrées d'audit affichent le nom de l'acteur (join avec `utilisateur`)
- [ ] CA-8 : Je peux filtrer le journal d'audit par utilisateur, table, action et plage de dates

### 6.3 Export Conformité
- [ ] CA-9 : L'export dossier étudiant génère un ZIP avec tous les documents et métadonnées, en < 30 secondes
- [ ] CA-10 : L'export inclut un manifeste JSON avec checksums SHA-256 pour chaque fichier
- [ ] CA-11 : L'export année académique génère un ZIP avec tous les CSV et index de documents, en < 2 minutes
- [ ] CA-12 : Tout export est journalisé dans `pister` avec `action = 'Exportation'`

### 6.4 Recherche
- [ ] CA-13 : Je peux rechercher par étudiant, année, thème, entreprise, jury, statut et mention
- [ ] CA-14 : Les résultats sont paginés et triables par date
- [ ] CA-15 : Je peux exporter les résultats de recherche en CSV

### 6.5 Transition d'Année
- [ ] CA-16 : Le processus de clôture copie tous les documents dans la structure `archives/YYYY-YYYY/<num_etu>/`
- [ ] CA-17 : Les sidecars JSON sont générés avec checksums SHA-256
- [ ] CA-18 : Le manifeste de l'année est généré avec les statistiques récapitulatives
- [ ] CA-19 : L'opération est journalisée dans `pister` avec `action = 'Clôture année'`
- [ ] CA-20 : Les données opérationnelles restent intactes (pas de déplacement de lignes BD)

### 6.6 Intégrité
- [ ] CA-21 : Je peux lancer une vérification des checksums et obtenir un rapport des fichiers corrompus/manquants
- [ ] CA-22 : Le rapport de réconciliation identifie les documents présents dans la BD mais absents du filesystem, et vice-versa

---

## 7. Exclusions et Limites

### 7.1 Limitations Acceptées
- **Audit granulaire** : La table `pister` ne stocke pas l'ID de l'objet ni les valeurs avant/après. La corrélation se fait par fenêtre temporelle.
- **Versionnement** : Pas de versionnement explicite des documents autres que les rapports (`rapport_etudiants.version`). Les CR sont versionnés par date.
- **Recherche plein texte** : Limitée aux colonnes indexées existantes. Pas de moteur de recherche full-text dédié.

### 7.2 Risques Identifiés
| Risque | Impact | Mitigation |
|--------|--------|------------|
| Tables sans colonne `actif` | Impossible de soft-delete | Filtrage applicatif, jamais de suppression physique |
| `pister` sans ID objet | Corrélation audit imprécise | Fenêtre temporelle + contexte métier |
| Volume documents important | Temps export élevé | Archivage incrémental, pas re-complet à chaque fois |
| Données historiques incomplètes | Dossiers partiels | Signaler les étapes manquantes dans la timeline |

---

## 8. Phasage d'Implémentation

Le déploiement du système d'archivage se fait en 5 phases sur environ 8 semaines :

| Phase | Objectif | Durée estimée |
|-------|----------|---------------|
| 1. Fondations | Requêtes historiques, états cycle de vie, conventions d'archive | 2 semaines |
| 2. Historisation Documents | Copie et organisation des documents dans l'arborescence d'archive | 2 semaines |
| 3. Renforcement Audit | Standardisation de la journalisation sur toutes les opérations | 1 semaine |
| 4. Reporting et Recherche | Recherche multi-critères, tableaux de bord analytiques | 2 semaines |
| 5. Conformité | Exports légaux, vérification intégrité, procédures rétention | 1 semaine |

**Détail des livrables par phase** : Voir **Annexe C**.

---

## 9. Métriques de Suivi

| Métrique | Cible | Fréquence mesure |
|----------|-------|------------------|
| % dossiers étudiants consultables (toutes années) | 100% | Mensuelle |
| Temps moyen export dossier étudiant | < 30s | Par export |
| Couverture audit (actions critiques journalisées) | > 95% | Hebdomadaire |
| Intégrité documents (checksums valides) | 100% | Trimestrielle |
| Temps recherche historique | < 3s | Par recherche |
| Satisfaction utilisateur (perçue) | > 4/5 | Trimestrielle |

---

## 10. Glossaire

| Terme | Définition |
|-------|------------|
| **Sidecar** | Fichier JSON de métadonnées associé à un document archivé |
| **Manifeste** | Fichier récapitulatif d'une archive (année ou export) avec liste des documents et checksums |
| **Timeline** | Vue chronologique des événements du parcours d'un étudiant |
| **Cycle de vie** | Ensemble des 13 étapes du parcours étudiant (inscription → clôture dossier) |
| **Archivage logique** | Stratégie d'archivage qui conserve les données dans les tables métier et reconstruit l'historique par requêtes |
| **Checksum SHA-256** | Empreinte cryptographique pour vérifier l'intégrité d'un fichier |
| **pister** | Table d'audit existante dans CheckMaster |

---

## Annexe A — Mapping Complet des Tables (62 tables explorées)

### Tables du périmètre archivage (utilisées directement)

| # | Table | Rôle archivage | Présence dans PRD |
|---|-------|----------------|-------------------|
| 1 | `etudiants` | Identité racine étudiant | §3.3 |
| 2 | `inscriptions` | Historique inscription | §3.3 |
| 3 | `annee_academique` | Bornage temporel | §3.3 |
| 4 | `rapport_etudiants` | Versions rapport | §3.3 |
| 5 | `deposer` | Événement dépôt | §3.3 |
| 6 | `valider` | Décision validation | §3.3 |
| 7 | `candidature_soutenance` | Candidature soutenance | §3.3 |
| 8 | `programmer_soutenance` | Événement soutenance | §3.3 |
| 9 | `enseignant_jury` | Composition jury | §3.3 |
| 10 | `enseignants` | Identité jury/encadrant | §3.3 |
| 11 | `evaluer` | Scores détaillés | §3.3 |
| 12 | `critere_evaluation` | Critères d'évaluation | §3.3 |
| 13 | `notes` | Résultat final | §3.3 |
| 14 | `compte_rendu` | Compte rendu délibération | §3.3 |
| 15 | `utilisateur` | Traçabilité acteur audit | §3.3 |
| 16 | `pister` | Journal d'audit | §3.3, §4.2 |
| 17 | `entreprises` | Dimension employeur | §3.3 |
| 18 | `informations_stage` | Contexte stage | §3.3 |
| 19 | `maitre_de_stage` | Maître de stage | §3.3 |
| 20 | `affecter` | Affectation encadrant | §3.3 |
| 21 | `qualite_jury` | Rôles jury | §3.3 |
| 22 | `bareme_critere` | Barème par critère | §3.3 |
| 23 | `compte_rendu_rapport` | Lien CR-rapport | §3.3 |
| 24 | `rendre` | Transmission CR | §3.3 |
| 25 | `session` | Session académique | §3.3 |
| 26 | `salles` | Salles de soutenance | §3.3 |
| 27 | `domaine` | Domaine de spécialité | §3.3 |
| 28 | `specialite` | Spécialité | §3.3 |
| 29 | `personnel_admin` | Personnel administratif | §3.3 |

### Tables système/référentiel (exploitées indirectement)

| # | Table | Rôle | Usage archivage |
|---|-------|------|-----------------|
| 30 | `type_utilisateur` | Types d'utilisateurs | Jointure audit (rôle acteur) |
| 31 | `groupe_utilisateur` | Groupes d'utilisateurs | Jointure audit (permissions) |
| 32 | `fonctionnalites` | Fonctionnalités système | Référence navigation |
| 33 | `permissions` | Permissions par rôle | Pas d'usage archivage direct |
| 34 | `niveau_etude` | Niveaux d'étude | Jointure inscriptions |

### Tables de workflow (consultées pour contexte)

| # | Table | Rôle | Usage archivage |
|---|-------|------|-----------------|
| 35 | `affecter` | Affectation encadrants | Reconstruction historique supervision |
| 36 | `deposer` | Dépôt rapports | Événement dans timeline |
| 37 | `valider` | Validation rapports | Événement dans timeline |
| 38 | `rendre` | Transmission CR | Événement dans timeline |

### Tables hors périmètre archivage

| # | Table | Raison d'exclusion |
|---|-------|-------------------|
| 39 | `reclamations` | Gestion opérationnelle quotidienne |
| 40 | `messages` | Communication interne utilisateurs |
| 41-62 | Autres tables système | Configuration, logs techniques, tables de jointure |

---

## Annexe B — Structure Filesystem et Formats de Métadonnées

### B.1 Structure Canonique des Archives

```
ressources/uploads/archives/
 2025-2026/
 manifeste_annee.json
 CI0116311179/
 manifeste_etudiant.json
 inscription/
 fiche_inscription.pdf
 rapports/
 rapport_123_v1.pdf
 rapport_123_v1_metadata.json
 rapport_123_v2.pdf
 rapport_123_v2_metadata.json
 comptes_rendus/
 cr_45_20260315.pdf
 cr_45_metadata.json
 pv/
 pv_final_2025-2026.pdf
 exports/
 dossier_complet_2025-2026.zip
 CI0116311180/
 ...
```

### B.2 Format Fichier de Métadonnées (Sidecar JSON)

```json
{
 "student": "CI0116311179",
 "academic_year": "2025-2026",
 "source_table": "rapport_etudiants",
 "source_id": 123,
 "version": 2,
 "original_path": "ressources/uploads/rapports/oldname.pdf",
 "archived_at": "2026-03-15T10:45:00+00:00",
 "checksum_sha256": "a1b2c3d4..."
}
```

### B.3 Format Manifeste d'Année

```json
{
 "academic_year": "2025-2026",
 "generated_at": "2026-09-01T00:00:00+00:00",
 "generated_by": "system_archive",
 "total_students": 145,
 "total_documents": 580,
 "students": [
 {
 "num_etu": "CI0116311179",
 "nom": "KOUAME",
 "prenom": "Jean",
 "status": "soutenu",
 "mention": "Bien",
 "documents_count": 8
 }
 ]
}
```

### B.4 Format Manifeste Étudiant

```json
{
 "student": "CI0116311179",
 "nom": "KOUAME",
 "prenom": "Jean",
 "academic_year": "2025-2026",
 "generated_at": "2026-09-01T00:00:00+00:00",
 "lifecycle_status": "Archivé",
 "documents": [
 {
 "type": "rapport",
 "id": 123,
 "version": 2,
 "filename": "rapport_123_v2.pdf",
 "checksum_sha256": "a1b2c3d4...",
 "archived_at": "2026-09-01T00:00:00+00:00"
 },
 {
 "type": "compte_rendu",
 "id": 45,
 "filename": "cr_45_20260315.pdf",
 "checksum_sha256": "e5f6g7h8...",
 "archived_at": "2026-09-01T00:00:00+00:00"
 }
 ],
 "timeline_summary": {
 "inscription_date": "2025-10-01",
 "report_submitted": "2026-02-15",
 "report_validated": "2026-03-01",
 "defense_date": "2026-03-26",
 "final_grade_m1": 14.5,
 "final_grade_m2": 15.0,
 "mention": "Bien"
 }
}
```

---

## Annexe C — Détail du Phasage d'Implémentation

### Phase 1 — Fondations (2 semaines)
- Inventaire des colonnes date/statut utilisables
- Normalisation des appels à `AuditLog::logAction()`
- Définition des états de cycle de vie
- Conventions de nommage des dossiers d'archive
- Vues/requêtes historiques de base (timeline étudiant, reporting année)

**Livrables** : Requête dossier étudiant, requête timeline, requêtes reporting année, requêtes audit.

### Phase 2 — Historisation Documents (2 semaines)
- Structure dossiers année/étudiant
- Copie des rapports/CR existants dans l'arborescence d'archive
- Génération sidecars métadonnées + checksums
- Service de récupération version-aware

**Livrables** : Service d'archivage copie, service de récupération, rapport de réconciliation.

### Phase 3 — Renforcement Audit (1 semaine)
- Standardisation de tous les appels de journalisation
- Journalisation de toutes les opérations de cycle de vie
- Ajout journalisation archive/export/clôture

**Livrables** : Catalogue d'actions d'audit, tableaux de bord audit, exports audit.

### Phase 4 — Reporting et Recherche (2 semaines)
- Recherche archive multi-critères
- Tableaux de bord analytiques multi-années
- Exports analytiques (graphiques + CSV)

**Livrables** : UI de recherche archives, exports analytiques historiques.

### Phase 5 — Conformité (1 semaine)
- Export dossier complet étudiant
- Export année académique
- Export demande légale
- Scripts de vérification rétention/intégrité

**Livrables** : Centre d'export conformité, procédures rétention et intégrité.
