
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
