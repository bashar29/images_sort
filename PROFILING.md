# Performance Profiling Guide

Ce guide explique comment utiliser les outils de profiling pour analyser et optimiser les performances du projet `images_sort`.

## 📊 Profiling intégré (Built-in)

Le projet inclut un système de mesure de performance intégré qui génère automatiquement un rapport détaillé.

### Utilisation

Exécutez simplement le programme normalement :

```bash
cargo run --release -- -s /path/to/source -d /path/to/dest
```

À la fin de l'exécution, vous verrez deux rapports :
1. **Image Sorting Report** : Statistiques sur le tri (images, lieux, devices, etc.)
2. **Performance Report** : Métriques de performance détaillées

### Exemple de sortie Performance Report

```
╔═══════════════════════════════════════════════════════════╗
║              ⚡ Performance Report                        ║
╠═══════════════════════════════════════════════════════════╣
║ 📖 EXIF reads              : 1234                         ║
║    Total time              : 2.45s                        ║
║    Average per read        : 2ms                          ║
║                                                            ║
║ 🌍 Geocoding lookups       : 856                          ║
║    Cache hits              : 0 (0.0%)                     ║
║    Total time              : 12.30s                       ║
║    Average per lookup      : 14ms                         ║
║                                                            ║
║ 📁 File copies             : 1234                         ║
║    Total size              : 1250.50 MB                   ║
║    Total time              : 45.20s                       ║
║    Average per file        : 37ms                         ║
║    Throughput              : 27.67 MB/s                   ║
║                                                            ║
║ 📂 Directory creations     : 42                           ║
║    Total time              : 0.15s                        ║
║    Average per mkdir       : 3ms                          ║
║                                                            ║
║ ⏱️  Time breakdown:                                        ║
║    EXIF reading            : 4.1%                         ║
║    Geocoding               : 20.5%                        ║
║    File copying            : 75.3%                        ║
║    Directory creation      : 0.3%                         ║
╚═══════════════════════════════════════════════════════════╝
```

### Métriques collectées

- **EXIF reads** : Temps de lecture et parsing des métadonnées EXIF
- **Geocoding lookups** : Temps de conversion GPS → nom de lieu (avec taux de cache hit)
- **File copies** : Temps de copie, volume total, débit (MB/s)
- **Directory creations** : Temps de création de répertoires
- **Time breakdown** : Répartition du temps par opération (%)

---

## 🔥 Flamegraph (CPU Profiling)

Pour une analyse approfondie des hotspots CPU, utilisez `cargo-flamegraph`.

### Installation

```bash
# macOS
cargo install flamegraph

# Si vous rencontrez des erreurs, installez DTrace
# DTrace est normalement préinstallé sur macOS
```

### Utilisation

```bash
# Générer un flamegraph
cargo flamegraph --release -- -s /path/to/source -d /path/to/dest

# Cela génère un fichier flamegraph.svg
# Ouvrez-le dans un navigateur pour l'explorer
open flamegraph.svg
```

### Interprétation du flamegraph

- **Axe horizontal** : Ne représente PAS le temps, mais l'ordre alphabétique des fonctions
- **Largeur** : Proportion du temps CPU passé dans cette fonction
- **Hauteur** : Profondeur de la pile d'appels
- **Couleurs** : Aléatoires, pour distinguer visuellement les fonctions

**Cherchez** :
- Les grandes plateformes larges = fonctions qui consomment beaucoup de CPU
- Les zones à optimiser en priorité

---

## 📈 Profiling avec perf (Linux uniquement)

Si vous êtes sur Linux, vous pouvez utiliser `perf` pour un profiling plus détaillé.

```bash
# Compiler en mode release avec symboles debug
cargo build --release

# Profiler avec perf
perf record --call-graph dwarf ./target/release/images_sort -s /source -d /dest

# Analyser les résultats
perf report
```

---

## 🔍 Identifier les goulots d'étranglement

Après avoir exécuté le profiling, voici comment interpréter les résultats :

### 1. Rapport de performance intégré

Regardez le **Time breakdown** :

- **Geocoding > 40%** : Considérez d'ajouter un cache pour les coordonnées GPS
- **File copying > 80%** : Normal sur HDD, c'est le goulot I/O
- **EXIF reading > 30%** : Opportunité de parallélisation
- **Directory creation > 5%** : Considérez un cache des répertoires créés

### 2. Débit de copie (Throughput)

Sur NAS HDD, attendez-vous à :
- **Lecture séquentielle** : 80-150 MB/s
- **Écriture séquentielle** : 60-120 MB/s
- **Accès réseau (NFS/SMB)** : 30-80 MB/s

Si votre throughput est **significativement plus bas**, c'est un signe de :
- Trop de parallélisme (thrashing du disque)
- Fragmentation
- Problèmes réseau (pour NAS distant)

### 3. Temps moyen par opération

Comparez vos temps avec ces références :

| Opération | Temps acceptable | Temps problématique |
|-----------|------------------|---------------------|
| EXIF read | 1-5 ms | > 20 ms |
| Geocoding lookup | 5-15 ms | > 50 ms |
| File copy (1MB) | 10-50 ms (HDD) | > 200 ms |
| Directory creation | 1-10 ms | > 50 ms |

---

## ✅ Optimisations implémentées

Les optimisations suivantes ont déjà été implémentées dans le code :

### **Phase 1 : Caches**

1. **✅ Cache geocoding (LRU)** :
   - Cache LRU de 1000 entrées pour les résultats de reverse geocoding
   - Précision ~11m (arrondi à 4 décimales)
   - Gain estimé : **50-90%** de réduction des lookups pour photos groupées géographiquement
   - Le rapport de performance affiche le **taux de cache hit**

2. **✅ Cache création répertoires** :
   - HashSet des répertoires déjà créés
   - Évite les appels `mkdir` redondants (important sur NAS)
   - Gain estimé : **20-40%** de réduction des appels système

### **Phase 2 : Parallélisation**

3. **✅ Parallélisation modérée (4 threads)** :
   - Traitement parallèle des images avec `rayon`
   - Limité à **4 threads** pour éviter le thrashing sur HDD
   - Compteurs atomiques pour éviter la contention
   - Gain estimé : **2-3x** sur la partie EXIF + geocoding

4. **✅ Reporting thread-safe** :
   - Compteurs atomiques (`AtomicU32`) pour les statistiques simples
   - RwLock uniquement pour les structures complexes (HashMap, Vec)
   - Élimine la contention lors du traitement parallèle

### **Prochaines optimisations potentielles**

Si le profiling révèle d'autres goulots :

1. **Buffer I/O** : Optimiser les buffers de copie pour le NAS (actuellement non nécessaire)
2. **Parallélisation adaptative** : Ajuster automatiquement le nombre de threads selon le type de stockage
3. **Cache géographique amélioré** : Utiliser des k-d trees pour des recherches encore plus rapides

---

## 📝 Notes

- Le profiling intégré a un overhead minimal (< 1%)
- Les mesures sont précises au niveau microseconde
- Pour des mesures reproductibles, exécutez plusieurs fois et faites la moyenne
- Le mode `--release` est recommandé pour des mesures réalistes
