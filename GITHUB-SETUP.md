# GitHub setup — _davSHARE

La release v26.10.2 segue lo standard `_davstudios` con versioning `YY.M.REVISIONE`.

## GitHub Desktop

Summary:

`_davSHARE v26.10.2`

La Description del commit deve contenere entrambe le sezioni `🇮🇹` e `🇺🇸`. Il workflow GitHub Actions legge automaticamente il body del commit associato al tag e lo usa come descrizione della GitHub Release. Se una delle due sezioni manca, la pubblicazione viene interrotta.

Dopo il commit esegui **Push origin**.

## Tag release

```bash
git tag -a v26.10.2 -m "Release _davSHARE v26.10.2"
git push origin v26.10.2
```

Il tag deve essere coerente con le versioni dichiarate in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` e `src-tauri/tauri.conf.json`.
