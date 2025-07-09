

# 各バージョンのazalea bot作成用リポジトリ


# AZALEAのコミット取り込み
GitHub Actions で以下を実行

1. azalea(https://github.com/azalea-rs/azalea)の最新コミットを取得 もしくは ワークフロー引数でコミットIDを指定
2. codegen/LATEST_AZALEA_OID と値が同じなら終了
3. azaleaのCargo.tomlのworkspace.package.version(e.g. 0.13.0+mc1.21.7)からマイクラのバージョン(1.21.7 = MC_VERSION)を取得

4. versions/{MC_VERSION} が存在しないなら {
    - versions/{codegen/LATEST_MC_VERSION} を versions/{MC_VERSION} にコピー
    - echo {MC_VERSION} > codegen/LATEST_MC_VERSION
}

5. versions/{MC_VERSION}/Cargo.toml の azalea関係のrevを更新
6. versions/{MC_VERSION}/Cargo.toml の anyhow と tokio のバージョンを azaleaのCargo.lock と同じにする
7. versions/{MC_VERSION}/Cargo.lock を azaleaのCargo.lock と同じにする
8. versions/{MC_VERSION}/rust-toolchain をazaleaのコミット日と同じnightlyにする
9. rust-toolchain に応じたrustupを導入し、cargo update を実行 (これでCargo.lockの中身が適切なものになる) (失敗した場合WEBHOOKで通知しつつ続行)
10. コミットしてプッシュ
11. ビルドとリリースの作成 ワークフローをトリガー

# ビルドとリリースの作成
GitHub Actions で以下を実行

1. ワークフロー引数でビルド対象のMC_VERSIONを指定
2. MC_VERSIONのリリースがない場合作成

3. [ win mac linux ] * [ x64, arm ] でそれぞれ実行 {
    - versions/{MC_VERSION}/rust-toolchain の rust を導入
    - ビルドを実行
    - Artifactとして保存
}

4. リリースにアーティファクトを追加/同名更新
