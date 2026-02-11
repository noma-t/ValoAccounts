# valo-accounts

複数のValorantアカウントを簡単に管理・切り替えができるデスクトップアプリケーション

## インストール方法

1. [GitHub Releases](https://github.com/noma-t/ValoAccounts/releases) から最新版の `.exe` ファイルをダウンロード
2. ダウンロードした `.exe` ファイルをダブルクリックして起動
3. アプリが起動すればインストール完了（インストーラー不要）
> [!NOTE]
> アプリを起動すると、アプリと同じフォルダーにデータベースファイル(data.db)が生成されます


## 機能

- **アカウント管理**: RiotID・タグライン・ログイン情報の登録・編集
- **ワンクリック切り替え**: アカウント選択後、ボタン1つで Valorant を切り替え起動
- **ランク自動取得**: HenrikDev API v3 を通じて現在のランクとレベルを自動更新
- **クリップボードコピー**: RiotID・ユーザー名・パスワードをワンクリックでコピー
- **tracker.gg 連携**: ボタン1つでアカウントの tracker.gg プロフィールを開く
- **プロセス管理**: Valorant・Riot Client の起動確認とワンクリック強制終了

## 設定

- **RiotClientService.exe Path**: `RiotClientService.exe` のパス
- **Riot Client Data Path**: デフォルトのDataパス
- **Account Data Path**: このアプリによって生成されるDataが配置されるパス
- **Region**: 地域
- **Henrikdev API Key**: ランクの自動取得に使用されるAPIのキー

## Henrikdev APIキーの取得

1. https://api.henrikdev.xyz/dashboard/ にアクセス
2. Discordと連携
3. [api-keys](https://api.henrikdev.xyz/dashboard/api-keys) にアクセス
4. Create First Keyをクリック
5. アプリ名・説明・アクセス層を選択し、Generate Keyをクリック
6. ACCESS TOKENをコピー

## システム要件

- **OS**: Windows 10 以降
- **Valorant**: インストール済み
- **Riot Client**: インストール済み

## 使用上の注意

- パスワードは Windows DPAPI によって暗号化されてローカルに保存されます（他の PC では復号不可）
- データベースファイルを削除すると、登録済みのアカウント情報もすべて削除されます

## 技術仕様

### 技術スタック

| 区分                       | 内容                                  |
|----------------------------|---------------------------------------|
| フロントエンド             | React 19 + TypeScript + Tailwind CSS  |
| デスクトップフレームワーク | Tauri 2                               |
| バックエンド               | Rust                                  |
| データベース               | SQLite                                |
| ランク取得                 | HenrikDev API v3                      |

### セキュリティ

- パスワードは [Windows DPAPI](https://learn.microsoft.com/en-us/windows/win32/seccng/cng-dpapi) でOSレベルの暗号化を適用して保存
- すべてのデータはローカルのみに保存され、外部サービスへの送信は行われない（ランク取得APIを除く）
