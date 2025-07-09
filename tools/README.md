
azaleaの更新に合わせてBOTの内容を更新して再ビルドをGitHub actionsから行いたい

やりたいこと

# botを更新する

-- ここからgithub actionで記述
1. このリポジトリをクローンする

-- ここからrustのtoolで実行
1. bot/Cargo.tomlをパースして、azaleaのrev(CURRENT_REV)を得る
2. ./azalea-tempにazaleaをクローン
3. azaleaのコミット履歴からCURRENT_REVの次のコミット(NEXT_REV)を探し出してチェックアウト
4. azalea/Cargo.tomlのworkspace.package.versionをパースして対応するマイクラのバージョン(MC_VERSION)を取得する
5. NEXT_REVがない場合(最新の場合)は終了
6. bot/Cargo.tomlを更新
7. azalea/Cargo.lockをbot/Cargo.lockにコピー
8. botでcargo updateを実行
9. 変更内容をコミット
-- ここまでrustのtoolで実行

9. mainにpush
-- ここまでgithub actionで記述

# 各OSでビルドする

各OSの各CPUアーキテクチャのマトリクスを作り並列実行

-- ここからgithub actionで記述
1. このリポジトリをクローンする
2. ビルドを実行 (nightly版)
3. ビルド結果をArtifactに格納 `flex-update-mc-bot-{MC_VERSION}-{os}-{arch}.{ext}`
-- ここまでgithub actionで記述

# リリースを作成更新する

-- ここからgithub actionで記述
1. 各OSでのビルド結果を{MC_VERSION}を含めたリリースとして公開する
2. すでに公開済みの場合はファイルを上書きする
-- ここまでgithub actionで記述


# ビルドメモ
cargo run -p tools --bin update-bot -- --next-rev 118f649cf7a6e401ee2eccd385b04a5478fdd311 --mc-version 1.19.3
cargo run -p tools --bin build-bot -- --os linux --arch x64 --rust-toolchain nightly-2023-03-13



## 1.19.2
6f9ffccde3e9ffde57811db19dd184b16f56bc83
nightly-2023-09-13

## 1.19.3
118f649cf7a6e401ee2eccd385b04a5478fdd311
nightly-2023-03-13

## 1.19.4
587ff91f16a3cae0bfe89e6781ad519ad66980b6
nightly-2023-05-08

## 1.20.1
0c05b4cd4271e3194c9bb8a265f8cc771b0f512b
nightly-2024-07-09

# 1.20.2
70cc93719f8139884ae0e48e58bbd099fe723149
???

# 1.20.4
5a460f38710b410399cb6750ff803e42b5989d6f
nightly-2025-07-08

# 1.20.5
b55b8698186d6eb973aaa3c9e759c25aaba7e891
nightly-2024-07-08

# 1.20.6
f35ba028f66ea9137a4326432c05f9254d0c67ce
nightly-2024-07-08

# 1.21.1
dfcb7c30aa17849711f5bde595c00d5e807c2eb1
nightly-2025-07-08

# 1.21.3
ea5a1c1ec128cc1a33593c9d91ef758c3fb73e16

# 1.21.4
8af265e48bf9f3d5263c074d034770e4216bb3f3

# 1.21.5
319d144995e0ca635806941cbb5d6ceaf0fcf515

# 1.21.6
a060b739158d9ff2cc3d7ecb13e79de091f1f055

# 1.21.7
ebc2e0c067d8b2c901ae02e032159e2c80eac7bc
