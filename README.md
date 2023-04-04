# murasaki: Nostr to Speech

⚠このソフトウェアはα版です⚠

[VOICEVOX](https://voicevox.hiroshiba.jp/) を利用したタイムライン読み上げツールです。

指定したリレーのグローバルタイムライン、または指定した公開鍵でフォローしているユーザのタイムラインを読み上げます。

## つかいかた

1. Rust をインストールします。

https://www.rust-lang.org/learn/get-started

以下のバージョンで動作しています。

2. [VOICEVOX](https://voicevox.hiroshiba.jp/) をインストールします。
[Engine](https://github.com/VOICEVOX/voicevox_engine) だけのインストールでも動作します。

3. 依存パッケージをインストールします。

    * Ubuntu Linux の場合: `sudo apt-get install -y build-essential libasound2-dev pkg-config libssl-dev`

4. このリポジトリをクローンします。

```
git clone https://github.com/darashi/murasaki.git
cd murasaki
```

5. `config.toml.example` を編集し、 `config.toml` として保存します。

6. ビルドして実行します。

```
cargo run --release
```

## 設定

`config.toml` で設定を変更できます。

VOICEVOX が起動している状態で http://localhost:50021/speakers を開くと、利用可能な音声合成エンジンの一覧が表示されます。`speaker` に `id` を指定してください。

`pubkey` に自分の公開鍵を設定すると、フォローしているユーザの note を読み上げます。
`pubkey` の指定を空にするとリレーから届くすべての note を読み上げます。

## 注意事項

このソフトウェアは、VOICEVOX の音声合成エンジンを利用します。
[VOICEVOX 利用規約](https://voicevox.hiroshiba.jp/term/) と各音声合成ライブラリの規約を遵守してください。

## 作者の動作確認環境

Ubuntu Linux 22.10

```
❯ rustc --version
rustc 1.68.0 (2c8cc3432 2023-03-06)
```
