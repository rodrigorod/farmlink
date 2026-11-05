# Cahier des Charges Fonctionnel - Front-end : FS25 Modpack Manager

## 1. Direction Artistique & Design System (Thème FS25)
* **Philosophie visuelle :** Éviter l'effet "SaaS générique / AI slop". S'inspirer des cockpits de machines agricoles modernes, des tableaux de bord industriels et de l'ambiance GIANTS Software (propre, robuste, technique).
* **Stack UI :** **Vue.js 3 (Composition API)** + **Tailwind CSS** + **Shadcn-vue** (pour des composants sur-mesure et non préfabriqués).
* **Palette de couleurs :**
  * **Fonds :** Ardoise très sombre / Noir mat (`#121417`, `#1a1d20`).
  * **Accents :** Vert agricole vif (`#438E33` / style John Deere / Case IH), Orange alerte (`#E67E22`) pour les versions obsolètes.
  * **Textes :** Blanc cassé (`#F3F4F6`) pour la lisibilité, gris technique (`#9CA3AF`) pour les métadonnées.
* **Typographie :** Polices sans-serif nettes (ex: *Inter* ou *Barlow*) avec des chiffres monospacés pour les versions (`v1.2.0.0`) et les tailles de fichiers.

---

## 2. Architecture des Vues & Pages (Routing)

### A. Espace Public & Authentification (`/auth`)
* **Page de Connexion / Inscription :** 
  * Connexion sécurisée par token (JWT fourni par le backend Go).
  * Gestion des rôles (Admin de serveur vs. Joueur simple).

### B. Dashboard Principal (`/dashboard`)
* **Vue d'ensemble des serveurs :**
  * Liste des serveurs multijoueurs FS25 auxquels l'utilisateur a accès.
  * Indicateur de statut rapide pour chaque serveur : 
    * 🟢 *Synchronisé / Prêt à jouer*
    * 🟠 *Mise à jour requise*
    * 🔴 *Non connecté / Erreur de version*
  * Bouton rapide "Rejoindre le serveur" (lance l'app compagnon Rust ou copie l'IP).

### C. Détail du Serveur & Modpack (`/servers/:id`)
* **Vue Joueur (La liste des mods) :**
  * Tableau ou grille claire listant tous les mods requis par le serveur.
  * **Colonnes / Infos par mod :**
    * Miniature / Icône du mod (extraite du modDesc.xml si disponible, ou icône générique de catégorie).
    * Nom du mod & Nom du fichier (`.zip`).
    * Version requise par le serveur (ex: `1.1.0.0`).
    * Statut par rapport au client local (Vérifié via l'app Rust : *À jour*, *Manquant*, *Version différente*).
    * Lien de téléchargement direct (S3).
  * **Bouton d'action global :** "Synchroniser tout via l'application compagnon".

### D. Panneau d'Administration du Modpack (`/servers/:id/admin`)
* *Réservé aux rôles Administrateur / Propriétaire du serveur.*
* **Zone de Glisser-Déposer (Drag & Drop) :**
  * Upload de nouveaux fichiers `.zip` de mods.
  * *Traitement instantané par le backend Go :* Lecture du `modDesc.xml`, calcul du hash MD5/SHA-256, extraction automatique de la version et envoi vers le stockage S3.
* **Gestion du Manifeste :**
  * Tableau de réarrangement ou de suppression des mods du serveur.
  * Activation / Désactivation temporaire d'un mod sans le supprimer de la base de données.
  * Génération de la version du modpack (ex: v2.4 -> déclenche l'alerte de mise à jour pour tous les joueurs).

### E. Paramètres & Téléchargement de l'App Compagnon (`/settings`)
* **Lien de téléchargement :** Section dédiée pour télécharger l'application cliente en **Rust** (Windows / Linux) en fonction de l'OS de l'utilisateur.
* **Configuration du chemin local :**
  * Guide visuel pour indiquer à l'application Rust où se trouve le dossier des mods FS25 (`.../My Games/FarmingSimulator2025/mods/`).
  * Clé d'API personnelle / Token de liaison entre le site web et l'application desktop Rust.

---

## 3. Expérience Utilisateur & États de l'Application (UX States)

* **États de chargement (Skeletons) :** Utilisation de skeletons aux formes géométriques strictes (pas d'animations de vagues fluides trop "corporate", préférer des clignotements discrets ou des barres de progression de type terminal/industriel).
* **Notifications (Toasts) :**
  * Alertes claires en cas de succès d'upload, d'échec de lecture d'un `modDesc.xml` corrompu, ou de désynchronisation de version.
* **Gestion du mode hors-ligne / Connexion App Rust :**
  * Indicateur visuel permanent dans la barre de navigation indiquant si l'application compagnon Rust est connectée au site web via un protocole local (WebSocket/IPC).
