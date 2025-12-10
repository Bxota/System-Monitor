# monitor_app

Petit moniteur système (CPU, RAM, réseau) en Rust + iced.

## Lancer en dev

```sh
cargo run
```

## Packager en .app macOS

1. Installer l’outil de bundling (une seule fois) :
   ```sh
   cargo install cargo-bundle
   ```
2. Construire le bundle release :
   ```sh
   make bundle
   # ou directement : cargo bundle --release
   ```
   Le fichier est généré dans `target/release/bundle/osx/monitor_app.app`.
3. Ouvrir le bundle :
   ```sh
   make run-bundle
   # ou open target/release/bundle/osx/monitor_app.app
   ```

### Icône personnalisée (optionnel)
- Placez un fichier `AppIcon.icns` dans un dossier `resources/` à la racine.
- Ajoutez `icon = "resources/AppIcon.icns"` dans la section `[package.metadata.bundle]` ci-dessous.

### Signature / notarisation (optionnel)
Pour une distribution sans alerte Gatekeeper :
- Signez : `codesign --deep --force --options=runtime --sign "Developer ID Application: VotreNom" target/release/bundle/osx/monitor_app.app`
- Notarisez : `xcrun notarytool submit target/release/bundle/osx/monitor_app.app --apple-id <id> --team-id <team> --password <app-specific>`

## Métadonnées bundle (ajoutées dans Cargo.toml)
Ajoutez ce bloc pour nom et icône (facultatif si vous voulez juste tester) :
```toml
[package.metadata.bundle]
identifier = "com.example.monitor_app"
name = "System Monitor"
# icon = "resources/AppIcon.icns"
```
