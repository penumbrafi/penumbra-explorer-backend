# Deploying the Penumbra explorer backend

The release binary is built in GitHub Actions from `main` or a tag and shipped
to CT1105 (`penumbra-web2`) over ssh. Nothing is compiled on the host — the
old `cargo build` in `/opt/penumbra-explorer-backend` is retired.

| workflow | what it does |
| --- | --- |
| `.github/workflows/ci.yml` | `cargo fmt --check`, clippy `-D warnings`, `cargo test` |
| `.github/workflows/deploy.yml` | release build -> `penumbra-explorer-<sha>.tar.zst` -> release (tags) -> deploy |

Both build inside `rust:1.89-bookworm` (the toolchain pinned by
`rust-toolchain.toml`). This matters: `ubuntu-latest` ships glibc 2.39 while
CT1105 is Debian bookworm with glibc 2.36, so a binary built on the bare
runner would abort at startup with a `GLIBC_2.3x not found` error. The
container also supplies the `libpq` that diesel links against.

## Release layout on the host

```
/opt/penumbra-explorer-backend/
  releases/<git-sha>/penumbra-explorer   the binary, plus BUILD_INFO
  current -> releases/<git-sha>
  .env                                   host-owned, never touched by CI
```

The deploy runs `--help` on the new binary before flipping `current`, so a
binary that cannot even load its shared libraries never reaches a restart.
Then: flip `current` atomically, prune to the five newest releases, restart
`penumbra-explorer.service`, smoke-test `http://127.0.0.1:9000/`.
`rsync --delete` only runs inside the new release directory, so `.env` and the
database survive untouched.

Migrations are embedded in the binary (`diesel_migrations::embed_migrations!`)
and run at startup, so nothing extra is shipped. `genesis.json` is read from
the absolute path in `GENESIS_JSON` (`/opt/penumbra-shared/genesis.json` on the
host) and is likewise not part of the artifact.

## Transport

CT1105 has sshd but only an internal address (`10.6.78.85`), so Actions uses
`ProxyJump` through the bkk06 hypervisor into a forwarding-only account. See
`.github/actions/ssh-deploy`.

## Secrets to create

GitHub **Environment** `production`, with these environment secrets:

| secret | value |
| --- | --- |
| `DEPLOY_SSH_KEY` | ed25519 private key for the deploy account |
| `DEPLOY_HOST` | public address of bkk06 (`160.22.180.6`) |
| `DEPLOY_CT` | CT1105 on the internal network (`10.6.78.85`) |
| `DEPLOY_KNOWN_HOSTS` | pinned host keys, below |

```
160.22.180.6 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMFjR7GW0By58m5FH+OBZ95VBB5ojplZa8C5UmjV731b
10.6.78.85 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBAWg1G4VempqSlmJtB+1XItpF7fHD8x+A3SBgJA0VQ1
```

No build-time variables are needed: everything is read at runtime from the
host `.env` and the `-g https://penumbra.rotko.net` flag in the unit.

## One-time host setup

The jump account on bkk06 and the `web` account in CT1105 are shared with the
other penumbrafi repos — set them up once, following `penumbrafi/web`'s
`deploy/README.md` (that README also covers `apt-get install rsync zstd`,
which CT1105 needs and does not have). In addition, inside CT1105:

```sh
install -d -o web -g web /opt/penumbra-explorer-backend/releases

# the unit's restart is already covered by /etc/sudoers.d/penumbra-deploy
install -m 644 deploy/systemd/penumbra-explorer.service /etc/systemd/system/
systemctl daemon-reload
```

`/opt/penumbra-explorer-backend/{.env,target,src,Cargo.*}` stay as they are
until the first CI deploy is verified; afterwards the source tree and
`target/` (a few GB) can be deleted, leaving `.env`, `releases/` and `current`.

Then add the vhost from `deploy/nginx-penumbra.fi.conf.example` in CT1102.

## Known unverified

* GitHub-hosted runner reachability to bkk06 `:22`.
* `cargo clippy -D warnings` and `cargo test --all` have not been run here;
  the previous pipeline used clippy pedantic, which may or may not be clean on
  `main` today. If CI is red on arrival, relax the clippy step rather than
  blocking deploys.
* The `-g https://penumbra.rotko.net` gRPC endpoint is copied verbatim from the
  running unit; switch it when the penumbra.fi RPC name exists.
