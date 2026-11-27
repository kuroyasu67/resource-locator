# 開発ツール

本文書は、組織の[ソフトウェア開発標準](https://github.com/dncinc-ict/software-development-standards)に従って、本リポジトリで定める設定を記述する。
対象は、標準と異なる運用、プロジェクトの種類と版、Rustの版、リンターとフォーマッター、Gitのフック、テスト、CI、依存関係の自動更新およびクレートの版の指定である。
Gitのフックの導入は、[Gitのフック](#gitのフック)に記述する。

## 標準と異なる運用

本リポジトリは、次の点で、標準と異なる運用をする。

- ブランチは、`main`ブランチのみで運用し、標準の[ブランチの運用](https://github.com/dncinc-ict/software-development-standards/blob/main/documents/branching.md)のgit-flow（`develop`ブランチ、`release`ブランチおよび`hotfix`ブランチ）は採用しない。
  したがって、ルールセットによるブランチとタグの保護、および`vX.Y.Z`のタグは、設定しない。
- 本クレートは、crates.ioに公開せず、利用者はGitのリポジトリから取り込む。
  したがって、crates.ioへの公開を前提とする、`include`によるパッケージに含めるファイルの指定、`cargo-semver-checks`による互換性を壊す変更の検出、およびcrates.ioへの公開のワークフローは、適用しない。

## プロジェクトの種類と版

本クレートは、利用者が依存して使うライブラリーである。
誤ってcrates.ioに公開しないように、`Cargo.toml`に`publish = false`を指定する。

- 版は、`Cargo.toml`の`[package]`の`version`に記述する。
- 公開するAPIは、`pub`で公開する関数、型、トレイトおよびマクロと、フィーチャー（`serde`）である。
  何が互換性を壊す変更に当たるかは、[Cargoの文書のSemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)に従う。
  1.0.0より前（`0.Y.Z`）は、互換性を壊す変更で`MINOR`を上げる。
- `rust-version`を上げることは、利用者のビルドに影響するため、`PATCH`では行わず、`MINOR`以上を上げる。

## Rustの版

本クレートでは、Rustの版を次の2か所で指定する。
2つの版は、目的が異なるため、揃えない。

| 指定 | 目的 |
| --- | --- |
| `rust-toolchain.toml`の`channel` | 開発者とCIが使うRustの版を固定し、`cargo fmt`やClippyの結果を揃える |
| `Cargo.toml`の`rust-version` | 本クレートの利用者がビルドできる、最小のRustの版（MSRV）を示す |

- `rustup`は、`rust-toolchain.toml`に指定した版のツールチェーンを、初めて`cargo`を実行したときに自動で導入する。
  導入するコンポーネントは、GitのフックとCIで使う`clippy`と`rustfmt`のみとする（`profile = "minimal"`）。
- 最新の安定版への追随は、[依存関係の自動更新](#依存関係の自動更新)で、`rust-toolchain.toml`を更新して行う。
- `rust-version`は、エディション2024を使える最も古い版の`1.85`とする。
  依存クレートの版を`Cargo.lock`に記録した版のまま、この版でライブラリーをビルドできることを確認した。
- `rust-version`を開発に使う版まで上げると、古いRustを使う利用者がビルドできなくなるため、`rust-version`は、必要になったときのみ手動で上げる。
- resolver 3は、`rust-version`の版でビルドできる依存クレートの版を、`Cargo.lock`に選ぶ。
  したがって、`Cargo.lock`を更新しても、MSRVでビルドできない版は取り込まれにくい。
- CIの`msrv`ジョブが、`rust-version`の版のツールチェーンで、ライブラリーをビルドできることを確認する。
  テストとサンプルの依存は、利用者のビルドに影響しないため、MSRVでは検査しない。
- `clippy.toml`に`msrv`を記述しない。
  Clippyは、`Cargo.toml`の`rust-version`を読み、MSRVで使えない機能を提案しない。

## リンターとフォーマッター

リンターの設定は、`Cargo.toml`の`[lints]`に記述する。
標準の既定（`unsafe_code = "forbid"`、`clippy::all`、`clippy::dbg_macro`、`clippy::todo`、`clippy::unimplemented`および`clippy::unwrap_used`）と、標準がライブラリーで有効にするとした次を、有効にする。

- `missing_docs`: 公開項目のドキュメントの漏れを検出する。
- `clippy::pedantic`: 公開するAPIの設計を、より厳しく検査する。

さらに、次を有効にする。

- `unreachable_pub`: クレートの外から使えない`pub`を検出し、公開する範囲を明確にする。
- `unused_qualifications`: 不要なパスの修飾を検出する。

テストでの`unwrap()`は、`clippy.toml`の`allow-unwrap-in-tests`で許可する。
警告を個別に抑止する場合は、`#[allow(...)]`ではなく`#[expect(..., reason = "...")]`を使い、理由を記述する。

フォーマッターは、Rustの公式のスタイルに従い、`rustfmt.toml`を置かずに、`rustfmt`の既定の設定で整形する。
`Cargo.toml`などのTOMLのファイルは、[Taplo](https://taplo.tamasfe.dev/)で整形し、設定は`taplo.toml`に記述する。
`taplo.toml`では、エントリーの`=`の位置を揃える（`align_entries`）。

## Gitのフック

Gitのフックは、[`prek`](https://github.com/j178/prek)で管理する。
設定は、`pre-commit`と互換性のある`.pre-commit-config.yaml`に記述し、`prek`に固有の`prek.toml`は使わない。
クローンした後に、[開発者が導入するツール](#開発者が導入するツール)を導入してから、`prek install`でフックを導入する。
`default_install_hook_types`に`pre-commit`と`pre-push`を指定しているため、1回の導入で、コミット時とプッシュ時の両方のフックが導入される。

| 時点 | ツール | 目的 |
| --- | --- | --- |
| コミット時 | `cargo fmt --check` | コードの書式を統一する |
| コミット時 | `taplo fmt --check` | TOMLのファイルの書式を統一する。整形は、`taplo fmt`で行う |
| コミット時 | `cargo metadata --locked` | `Cargo.toml`を変更して、`Cargo.lock`を更新し忘れたコミットを防ぐ |
| コミット時 | `cargo deny --locked check` | 依存クレートを検査する。`Cargo.toml`、`Cargo.lock`または`deny.toml`を変更した場合のみ実行する |
| コミット時 | `scripts/check-ci-tool-versions.sh` | CIの複数のジョブに記述したツールの版が、揃っていることを確認する |
| コミット時 | `gitleaks` | 秘密情報のコミットを防ぐ |
| コミット時 | `detect-private-key` | 秘密鍵のコミットを防ぐ |
| コミット時 | `check-yaml`、`check-json`、`check-toml`、`check-merge-conflict`、`check-case-conflict`、`check-added-large-files`、`fix-byte-order-marker`、`mixed-line-ending`、`end-of-file-fixer`、`trailing-whitespace` | 設定ファイルの構文の誤り、衝突を解消し忘れた印、大きなファイルおよび改行コードの誤りを防ぐ |
| コミット時 | `markdownlint-cli2` | Markdownのリンク切れや書式の誤りを検出する。設定は`.markdownlint-cli2.jsonc`に記述する |
| プッシュ時 | `cargo clippy --all-targets --all-features -- -D warnings` | 警告をエラーとして扱う |
| プッシュ時 | `cargo nextest run --all-features` | 単体テストを実行する |

- 使用されていない依存クレートの検出（`cargo machete`）は、フックに加えず、CIの`machete`ジョブで実行する。
- 本リポジトリには、統合テストがない。
  統合テストを追加した場合は、作業ブランチをリモートにプッシュする前に、必ず実行する。

### 開発者が導入するツール

フックとテストで使う、次のツールは、開発者が導入する。
CIで使うツールと同じ版を導入する。

| ツール | 版 | 用途 |
| --- | --- | --- |
| [`prek`](https://github.com/j178/prek) | 0.5.11 | Gitのフックの管理 |
| [`taplo`](https://taplo.tamasfe.dev/) | 0.10.0 | TOMLのファイルの書式の検査と整形 |
| [`cargo-nextest`](https://nexte.st/) | 0.9.152 | 単体テストの実行 |
| [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) | 0.22.0 | 依存クレートの検査 |
| [`cargo-machete`](https://github.com/bnjbvr/cargo-machete) | 0.10.1 | 使用されていない依存クレートの検出 |

```shell
cargo install prek --locked --version 0.5.11
cargo install taplo-cli --locked --version 0.10.0
cargo install cargo-nextest --locked --version 0.9.152
cargo install cargo-deny --locked --version 0.22.0
cargo install cargo-machete --locked --version 0.10.1
prek install
```

- Rustのツールチェーンは、`rust-toolchain.toml`の版を、`rustup`が自動で導入する。
- `gitleaks`と`markdownlint-cli2`は、`prek`が、`.pre-commit-config.yaml`に記述した版を自動で導入する。

## テスト

テストランナーには、[`cargo-nextest`](https://nexte.st/)を使う。
`cargo-nextest`は、テストを別々のプロセスで並行して実行するため、`cargo test`より速く、失敗したテストの出力が読みやすい。

```shell
cargo nextest run --all-features
```

`cargo-nextest`は、ドキュメントテストを実行しない。
したがって、ドキュメントテストは、`cargo test`で別に実行する。

```shell
cargo test --doc --all-features
```

`serde`のフィーチャーを有効にしない、既定のフィーチャーでも、テストが通ることを確認する。

```shell
cargo nextest run
cargo test --doc
```

テストは、プッシュ時のGitのフックとCIで実行する。

## CI

CIは、GitHub Actionsの`.github/workflows/ci.yml`で実行する。
`main`ブランチへの変更は、Pull Requestを経由するため、CIは`main`ブランチへのPull Requestで実行し、マージした後のブランチでは、同じ検査を繰り返さない。
同じPull Requestで新しいコミットをプッシュした場合は、古いコミットで実行中のCIを取り消す。
手動で実行するCIは、Pull Requestの検査と結果が食い違うことがあるため、`workflow_dispatch`では起動しない。
ランナーの不調などで失敗したCIは、Pull Requestの画面で、失敗した実行を実行し直す（「Re-run jobs」）。

Gitのフックは、手元で省略できる。
したがって、CIで同じ検査を実行し、CIを最終の関門とする。

| ジョブ | 内容 |
| --- | --- |
| `changes` | Pull Requestで変更したファイルから、検査のジョブを実行するかを判定する。`main`ブランチへの`push`では、検査のジョブを実行しない |
| `secrets` | `prek run gitleaks --all-files`で、シークレットを検出する。変更したファイルの判定にかかわらず、すべてのPull Requestで実行する |
| `hooks` | `prek run --all-files`で、コミット時の[Gitのフック](#gitのフック)を実行する。`gitleaks`と`cargo-deny`は、`secrets`ジョブと`deny`ジョブで実行するため、除く（`SKIP`） |
| `clippy` | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`を実行する |
| `docs` | ドキュメントの誤り（例: 存在しない項目へのリンク）を検出するため、`RUSTDOCFLAGS: -D warnings`で、`cargo doc --workspace --no-deps --all-features --locked`を実行する |
| `unit-tests` | `cargo nextest run`と`cargo test --doc`で、単体テストとドキュメントテストを、すべてのフィーチャーを有効にした場合と、既定のフィーチャーの場合の両方で実行する |
| `msrv` | `Cargo.toml`の`rust-version`の版で、`cargo check --lib`を、すべてのフィーチャーを有効にした場合と、既定のフィーチャーの場合の両方で実行する |
| `deny` | `cargo deny --locked check`で、依存クレートの勧告、ライセンス、使用を禁止するクレートおよび取得元を検査する |
| `machete` | `cargo machete`で、使用されていない依存クレートを検出する |
| `cache` | `main`ブランチへの`push`でのみ実行し、`cargo check`と`cargo build`で、検査のジョブが使うビルドのキャッシュを作り直す。検査はしない |
| `result` | 他のジョブが、成功したか、変更がないため省略されたことを確認する（`CI result`） |

- ドキュメントなど、次のファイルのみを変更したPull Requestでは、`changes`ジョブが判定し、`secrets`ジョブを除く検査のジョブを省略する。
  ワークフローの`paths`で省略しないのは、ワークフローが起動しないと、`CI result`が報告されないためである。
  省略しないファイルを1つでも含む変更では、すべての検査のジョブを実行する。
  - Markdownのファイル（`**/*.md`）。ドキュメントテストとして取り込むMarkdownのファイルはない
  - `.editorconfig`、`.markdownlint-cli2.jsonc`（または`.markdownlint-cli2.yaml`）、`.github/dependabot.yml`および`LICENSE-*`
- Rustのビルドのキャッシュは、Pull Requestの間で共有されず、デフォルトブランチ（`main`）のキャッシュのみを共有する。
  したがって、`Cargo.lock`または`rust-toolchain.toml`を変更したPull Requestを`main`ブランチにマージした場合に限り、`cache`ジョブのみを実行して、キャッシュを作り直す。
  検査のジョブは、キャッシュを保存しない（`Swatinem/rust-cache`の`save-if`）。
  `msrv`ジョブは、他のジョブと異なるツールチェーンを使うため、キャッシュのキーを分ける（`shared-key: msrv`）。
- 第三者のActionsは、タグを差し替えられても内容が変わらないよう、コミットのハッシュで指定する。
- ジョブごとに、`timeout-minutes`で実行時間の上限を設ける。
- `cargo`のコマンドには`--locked`を指定し、`Cargo.lock`と`Cargo.toml`が食い違う場合は失敗させる。
- `deny`ジョブは、Dependabotが`Cargo.lock`を更新しない間に公開された勧告も検出する。
- 本クレートは、crates.ioに公開しないため、`semver-checks`ジョブは実行しない（[標準と異なる運用](#標準と異なる運用)を参照）。
- ツールは、`taiki-e/install-action`で、次の版を導入する。

  | ツール | 版 | ジョブ |
  | --- | --- | --- |
  | `prek` | 0.5.11（`j178/prek-action`の`prek-version`） | `secrets`、`hooks` |
  | `taplo` | 0.10.0 | `hooks` |
  | `cargo-nextest` | 0.9.152 | `unit-tests` |
  | `cargo-deny` | 0.22.0 | `deny` |
  | `cargo-machete` | 0.10.1 | `machete` |

## 依存関係の自動更新

依存関係は、手動で追随すると更新が滞り、脆弱性の修正を取り込み遅れる。
したがって、GitHubの[Dependabot](https://docs.github.com/ja/code-security/dependabot)で、更新のPull Requestを自動で作成し、CIで検査してからマージする。
設定は、`.github/dependabot.yml`に記述する。

| 対象 | 更新の方法 |
| --- | --- |
| クレート | 間接的な依存を含めて更新する。`versioning-strategy`は既定の`auto`とし、`Cargo.toml`の版の範囲内の更新では`Cargo.lock`のみを更新し、範囲を超える更新では`Cargo.toml`と`Cargo.lock`を更新する。マイナーとパッチの更新は、1つのPull Requestにまとめる |
| Rustの版 | `rust-toolchain.toml`を更新する。`Cargo.toml`の`rust-version`（MSRV）は更新しない |
| GitHub Actions | コミットのハッシュと、コメントの版を更新する。マイナーとパッチの更新は、1つのPull Requestにまとめる |
| Gitのフック | `.pre-commit-config.yaml`のフックの版を更新する |

その他の方針は、次のとおりである。

- Dependabotは、`Asia/Tokyo`の時刻で毎週月曜日の0時に、更新を作成する。
- Dependabotは、公開から3日を経過していない版を取り込まない。
  公開の直後に取り下げられた版や、乗っ取られて公開された版を、取り込まないためである。
- セキュリティ更新は、`cargo`と`github-actions`のそれぞれで、1つのPull Requestにまとめる。
- Dependabotが作成したPull Requestへの対応の手順は、READMEの「依存関係の更新」に記述する。
- 新しい安定版のRustで追加されたリンターにより、Clippyの警告が出た場合は、Rustの版を更新するPull Requestで修正する。

### 手動で揃える箇所

Dependabotは、ワークフローの`prek-version`のような、Actionsの入力に書いたツールの版を更新しない。
したがって、次の版は、手動で更新する。

| 版 | 記述する箇所 | 更新の契機と確認 |
| --- | --- | --- |
| `prek`の版 | `.github/workflows/ci.yml`の`secrets`ジョブと`hooks`ジョブ | 手元の`prek`を更新したときに、揃えて更新する。`scripts/check-ci-tool-versions.sh`が、2つのジョブで揃っていることを確認する |
| `taplo`、`cargo-nextest`、`cargo-deny`および`cargo-machete`の版 | `.github/workflows/ci.yml` | 定期的に最新の版を確認して更新する。`taiki-e/install-action`が対応している版から選ぶ。`scripts/check-ci-tool-versions.sh`が、版を指定していることを確認する。更新したときは、[開発者が導入するツール](#開発者が導入するツール)の版も更新する |

## クレートの版の指定

本クレートは、利用者が依存して使うライブラリーである。
Cargoは、利用者のプロジェクトのすべての依存を通じて、1つのクレートにつき、互換性のある版を1つに揃える。
したがって、`Cargo.toml`の版の下限を高くすると、利用者が使える版の組み合わせを狭める。

- 版は、キャレット指定で書く。キャレット指定は、演算子を付けない指定である。
  - `*`による指定は、どの版でも受け入れるため、使わない（`cargo-deny`で禁止する）。
  - `=`による完全固定は、利用者の依存と衝突し、脆弱性の修正版も取り込めなくなるため、使わない。
  - 下限は、本クレートが使う機能を含む、最も古い版にする（例: `"1"`、`"0.3"`）。
    `Cargo.lock`の版に揃えて上げることはしない。
- 公開するAPIには、なるべく依存クレートの型を含めない。
  含めると、その依存クレートの`MAJOR`（1.0.0より前は`MINOR`）を上げる更新が、本クレートの互換性を壊す変更になるためである。
- `Cargo.lock`はリポジトリに含め、開発とCIで使う版を固定する。
  `Cargo.lock`は、利用者のビルドには使われない。
- 既定のフィーチャーを無効にする場合や、特定のフィーチャーを選ぶ場合は、理由をコメントに記述する。
  使うフィーチャーのみを有効にし、使わないフィーチャーは有効にしない。
- 使っていない依存は、`cargo machete`が検出するため、削除する。
  テストとサンプルのみで使うクレートは、`[dev-dependencies]`に置く。
