# Academ-IA Node — Local AI Worker

> Le Worker local d'Academ-IA. Installe-le sur le serveur de ton entreprise pour faire tourner des modèles IA en local, sans que tes données ne quittent jamais ton réseau.

---

## Qu'est-ce que c'est ?

**Academ-IA Node** est une application native légère (`.exe` sur Windows, `.dmg` sur macOS) qui :

- Télécharge et gère **Ollama** automatiquement (pas d'installation séparée)
- Crée un **tunnel sécurisé chiffré** vers `academ-ia.com`
- S'**enregistre automatiquement** dans le back-office Academ-IA
- Vérifie et applique les **mises à jour automatiquement** au démarrage
- Permet aux employés d'utiliser l'IA locale via leur navigateur, sans configuration

## Architecture

```
[Employé sur navigateur] → [academ-ia.com] → [Tunnel chiffré] → [Academ-IA Node]
                                                                        ↓
                                                               [Ollama local]
                                                               [Modèles IA]
                                                          (données 100% locales)
```

## Installation

### Windows
1. Télécharger `AcademIA-Node-Setup.exe` depuis la page de téléchargement
2. Lancer l'installateur (droits administrateur requis)
3. Saisir la **clé d'organisation** fournie par Academ-IA
4. Le Worker démarre et se connecte automatiquement

### macOS
1. Télécharger `AcademIA-Node.dmg`
2. Glisser l'application dans le dossier Applications
3. Lancer l'application et saisir la **clé d'organisation**
4. Autoriser les connexions réseau si demandé par macOS

## Modèles IA supportés

| Modèle | Taille | Usage recommandé |
| :--- | :--- | :--- |
| `llama3.3:8b` | 4.7 GB | Usage général, rapide |
| `llama3.3:70b` | 40 GB | Raisonnement complexe |
| `mistral:7b` | 4.1 GB | Rédaction, résumés |
| `qwen2.5:7b` | 4.4 GB | Multilingue, code |
| `phi4:14b` | 8.5 GB | Analyse, synthèse |
| `gemma3:9b` | 5.4 GB | Tâches générales |

Les modèles sont téléchargés à la demande depuis le panneau Admin Entreprise.

## Configuration requise

| Composant | Minimum | Recommandé |
| :--- | :--- | :--- |
| **RAM** | 8 GB | 16 GB |
| **GPU VRAM** | 4 GB (optionnel) | 8-16 GB NVIDIA/AMD |
| **Stockage** | 10 GB | 50 GB |
| **OS** | Windows 10 / macOS 12 | Windows 11 / macOS 14 |
| **Réseau** | Port 443 sortant | Port 443 sortant |

## Développement

### Prérequis
- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Lancer en développement
```bash
pnpm install
pnpm tauri dev
```

### Compiler manuellement
```bash
# Windows (depuis une machine Windows)
pnpm tauri build --target x86_64-pc-windows-msvc

# macOS (depuis une machine macOS)
pnpm tauri build --target x86_64-apple-darwin
pnpm tauri build --target aarch64-apple-darwin  # Apple Silicon
```

### Compilation automatique via GitHub Actions
Chaque push sur `main` déclenche automatiquement la compilation pour Windows et macOS.
Les binaires sont disponibles dans les **Releases** GitHub.

## Variables d'environnement

| Variable | Description |
| :--- | :--- |
| `ACADEMIA_API_URL` | URL de la plateforme Academ-IA (défaut: `https://acad-ia-78kiixzl.manus.space`) |
| `ACADEMIA_ORG_KEY` | Clé d'organisation (saisie lors de l'installation) |
| `OLLAMA_HOST` | Adresse d'Ollama (défaut: `http://localhost:11434`) |
| `TUNNEL_ENABLED` | Activer le tunnel sécurisé (défaut: `true`) |

## Sécurité

- Toutes les communications entre le Worker et academ-ia.com sont **chiffrées TLS 1.3**
- Les données des conversations **ne transitent jamais** par les serveurs Academ-IA
- La clé d'organisation est **stockée chiffrée** localement (AES-256)
- Le Worker ne peut être utilisé que par les membres de l'organisation autorisée

## Licence

Propriétaire — © 2025 Academ-IA. Tous droits réservés.
