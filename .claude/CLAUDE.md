# valo-accounts プロジェクト設定

Valorant の複数アカウント管理をサポートするデスクトップアプリケーション。

## プロジェクト概要

- **技術スタック**: Tauri 2 + React 19 + TypeScript + Rust
- **フロントエンド**: React + TypeScript + Vite
- **バックエンド**: Rust (Tauri)
- **データベース**: SQLite
- **対応OS**: Windows 10 以降

## 重要なルール

### 1. コード組織

- フロントエンド: `src/` ディレクトリに Components, Pages, Hooks, Utils を分類
- バックエンド: `src-tauri/src/` に Rust ロジック実装
- 小さなファイル多数が優先 (200-400 行、最大 800 行)
- 機能・ドメイン別に整理 (タイプ別ではない)

### 2. コードスタイル

- コード、コメント、ドキュメント内に絵文字は禁止
- **イミュータビリティ必須**: オブジェクト/配列の直接変更は禁止
- React での immutable patterns: `{...obj, key: value}` / `[...arr]` を使用
- `console.log` は本番コードに含めない (デバッグ時の一時的な使用のみ)
- try/catch による適切なエラーハンドリング
- Zod による入力検証

### 3. テスト

- TDD (Test-Driven Development): テストを先に書く
- 最小 80% カバレッジ
- ユーティリティ: ユニットテスト
- API/Tauri コマンド: インテグレーション テスト
- 重要フロー: E2E テスト (Playwright)

### 4. セキュリティ

- **ハードコードされたシークレット禁止**
- 環境変数で機密情報を管理 (.env)
- すべてのユーザー入力を検証
- パスワードは暗号化して SQLite に保存
- Tauri の allow リスト設定を厳密に

## ファイル構造

```
valo-accounts/
├── src/                      # React フロントエンド
│   ├── components/          # 再利用可能なUI コンポーネント
│   ├── pages/              # ページ/スクリーン
│   ├── hooks/              # カスタムReact フック
│   ├── lib/                # ユーティリティ関数
│   ├── types/              # TypeScript型定義
│   ├── App.tsx             # メインコンポーネント
│   └── main.tsx            # エントリーポイント
├── src-tauri/              # Tauri バックエンド (Rust)
│   ├── src/
│   │   ├── lib.rs         # Rust ライブラリ
│   │   └── main.rs        # Tauri アプリケーション
│   ├── Cargo.toml
│   └── tauri.conf.json    # Tauri 設定
├── public/                 # 静的ファイル
└── index.html             # HTML テンプレート
```

## 主な実装パターン

### Tauri コマンド (Rust → Frontend 通信)

```rust
// src-tauri/src/lib.rs
#[tauri::command]
pub fn get_accounts() -> Vec<Account> {
    // 実装
}
```

```typescript
// src/lib/tauri.ts
import { invoke } from '@tauri-apps/api/core'

export async function getAccounts() {
    return invoke('get_accounts')
}
```

### React コンポーネント

```typescript
// src/components/AccountList.tsx
import { useState, useEffect } from 'react'
import { getAccounts } from '../lib/tauri'

export function AccountList() {
    const [accounts, setAccounts] = useState([])

    useEffect(() => {
        getAccounts().then(setAccounts)
    }, [])

    return <div>{/* レンダリング */}</div>
}
```

### エラーハンドリング

```typescript
try {
    const result = await someOperation()
    return { success: true, data: result }
} catch (error) {
    console.error('Operation failed:', error)
    return { success: false, error: 'ユーザーフレンドリーなメッセージ' }
}
```

### 入力検証 (Zod)

```typescript
import { z } from 'zod'

const accountSchema = z.object({
    riotId: z.string().min(1),
    tag: z.string().min(1),
    email: z.string().email(),
    password: z.string().min(8)
})

const validated = accountSchema.parse(input)
```

## 環境変数

```bash
# .env (リポジトリに含めない)
TAURI_PRIVATE_KEY=          # Tauri署名キー
TAURI_KEY_PASSWORD=         # キーパスワード
```

## 開発コマンド

```bash
# フロントエンド開発サーバー
pnpm dev

# 全体開発モード (Tauri + Vite)
pnpm tauri dev

# ビルド
pnpm build

# Tauri ビルド
pnpm tauri build

# テスト実行
pnpm test
```

## Tauri 設定 (重要)

- `tauri.conf.json` の allowlist は最小限に設定
- 不要な API アクセスは許可しない
- Process API (プロセスキル): `allowlist.process` で有効化

## Git ワークフロー

- Conventional Commits:
  - `feat:` - 新機能
  - `fix:` - バグ修正
  - `refactor:` - リファクタリング (機能変更なし)
  - `docs:` - ドキュメント変更
  - `test:` - テスト追加・修正
  - `chore:` - 雑務・保守 (ビルド設定、依存関係更新など)
  - `perf:` - パフォーマンス改善
  - `ci:` - CI/CD 設定変更
  - `build:` - ビルドシステム変更
  - `style:` - コードスタイル変更 (フォーマット、セミコロンなど)
  - `revert:` - コミットの取り消し
- **コミットメッセージは常に英語で書く**
- **コミットメッセージはリリースノートに直接使用される**
  - ユーザーが読むことを意識した、分かりやすいメッセージを書く
  - `feat:` / `fix:` のコミットは特にリリースノートに記載される
- 変更が複数の関心事にまたがる場合は、**複数のコミットに分割してよい**
  - 例: 機能追加とバグ修正は別コミットにする
  - 例: Rust バックエンドの変更とフロントエンドの変更を分けてもよい
- メインへの直接コミットは禁止
- すべてのテストがパスしてから merge
- コミット前にローカルで検証

## 利用可能なスキル・コマンド

- `/tdd` - テスト駆動開発ワークフロー
- `/plan` - 実装計画作成
- `/security-review` - セキュリティチェック
- `/build-fix` - ビルドエラー修正

## 成功基準

- ✅ すべてのテスト合格 (80%+ カバレッジ)
- ✅ セキュリティ脆弱性なし
- ✅ コードは読みやすく保守しやすい
- ✅ ユーザー要件を満たしている
