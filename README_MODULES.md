# Architecture Modulaire - monitor_app

## Vue d'ensemble

Le projet utilise un système de **features optionnels** Cargo qui permet d'activer ou désactiver les modules indépendamment. Cela permet au code principal de fonctionner avec ou sans ces modules.

## Modules disponibles

### Modules de monitoring

- **`battery`** : Affichage du niveau et de l'état de la batterie (macOS)
- **`network`** : Monitoring du trafic réseau (download/upload en Mbps, total en GiB)
- **`disk`** : Affichage de l'utilisation du stockage disque

### Module interface

- **`widget`** : Active le widget compact pour la barre de menu (nécessite tray-icon)

## Configuration par défaut

```toml
[features]
default = ["battery", "network", "disk"]
```

Par défaut, tous les modules de monitoring sont activés pour une expérience complète.

## Exemples d'utilisation

### Compilation avec tous les modules (défaut)

```sh
cargo build --release
cargo run
```

### Compilation minimale (CPU + RAM uniquement)

```sh
cargo build --release --no-default-features
cargo run --no-default-features
```

L'application affichera uniquement :
- 💻 CPU (pourcentage d'utilisation)
- 🧠 RAM (pourcentage et utilisation mémoire)

### Compilation avec modules sélectionnés

```sh
# CPU, RAM + Batterie uniquement
cargo build --release --no-default-features --features battery

# CPU, RAM + Réseau + Stockage (sans batterie)
cargo build --release --no-default-features --features network,disk

# Toutes les combinaisons sont possibles
cargo run --no-default-features --features battery,network
```

### Widget avec modules spécifiques

```sh
# Widget avec tous les modules
cargo run --bin monitor_widget --features widget

# Widget minimal (CPU + RAM uniquement)
cargo run --bin monitor_widget --no-default-features --features widget

# Widget avec batterie uniquement
cargo run --bin monitor_widget --no-default-features --features widget,battery
```

## Avantages de l'architecture modulaire

### 1. Flexibilité
- Adaptez l'application à vos besoins spécifiques
- Activez uniquement les fonctionnalités dont vous avez besoin

### 2. Performance
- **Binaire plus léger** : Moins de code compilé
- **Compilation plus rapide** : Moins de dépendances à compiler
- **Runtime optimisé** : Pas de code mort

### 3. Portabilité
- Le module `battery` est spécifique à macOS
- Possibilité de désactiver les modules non supportés sur d'autres plateformes

### 4. Maintenance
- Code modulaire et organisé
- Chaque module est indépendant
- Tests et développement simplifiés

## Architecture du code

### Structure des modules (lib.rs)

```rust
// Module batterie (optionnel)
#[cfg(feature = "battery")]
pub mod battery {
    pub fn get_battery_info() -> (f32, bool) { ... }
}

// Fonction stub si le module est désactivé
#[cfg(not(feature = "battery"))]
pub fn get_battery_info() -> (f32, bool) {
    (100.0, false)  // Valeurs par défaut
}
```

### Application principale (main.rs / widget.rs)

```rust
// Champs conditionnels dans la structure State
struct State {
    cpu: f32,
    used_mem_mb: u64,
    total_mem_mb: u64,
    
    #[cfg(feature = "network")]
    networks: Networks,
    #[cfg(feature = "network")]
    down_mbps: f32,
    
    #[cfg(feature = "battery")]
    battery_percent: f32,
    // ...
}

// Affichage conditionnel dans la vue
#[cfg(feature = "network")]
{
    metrics_column = metrics_column.push(create_metric_row(
        "🌐 Réseau",
        format!("↓{:.1} ↑{:.1}", state.down_mbps, state.up_mbps),
        Color::from_rgb8(0x10, 0xb9, 0x81),
    ));
}
```

## Tester l'architecture modulaire

### Vérifier la compilation

```sh
# Tous les modules
cargo check

# Sans modules
cargo check --no-default-features

# Chaque combinaison
cargo check --no-default-features --features battery
cargo check --no-default-features --features network
cargo check --no-default-features --features disk
cargo check --no-default-features --features battery,network,disk
```

### Comparer la taille des binaires

```sh
# Version complète
cargo build --release
ls -lh target/release/monitor_app

# Version minimale
cargo build --release --no-default-features
ls -lh target/release/monitor_app
```

## Ajouter un nouveau module

Pour ajouter un nouveau module au système :

1. **Déclarer la feature dans Cargo.toml**
   ```toml
   [features]
   default = ["battery", "network", "disk", "nouveau_module"]
   nouveau_module = []
   ```

2. **Créer le module dans lib.rs**
   ```rust
   #[cfg(feature = "nouveau_module")]
   pub mod nouveau_module {
       pub fn get_data() -> DataType { ... }
   }
   
   #[cfg(not(feature = "nouveau_module"))]
   pub fn get_data() -> DataType {
       DataType::default()
   }
   ```

3. **Utiliser conditionnellement dans main.rs/widget.rs**
   ```rust
   #[cfg(feature = "nouveau_module")]
   use monitor_app::get_data;
   
   struct State {
       #[cfg(feature = "nouveau_module")]
       data: DataType,
   }
   ```

4. **Afficher conditionnellement dans la vue**
   ```rust
   #[cfg(feature = "nouveau_module")]
   {
       // Code d'affichage du module
   }
   ```

## Conclusion

Cette architecture modulaire rend le projet flexible, maintenable et optimisé. Chaque utilisateur peut compiler exactement ce dont il a besoin, et le code reste propre et organisé.
