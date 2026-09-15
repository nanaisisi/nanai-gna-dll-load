# nanai-gna-dll-rs

RustでIntel GNA (Gaussian & Neural Accelerator) のランタイムDLL/共有ライブラリを動的ロード（libloading）して使用するためのライブラリです。

## 特長

- **ビルド時リンク不要**: 実行時に動的ロードするため、開発環境に事前に GNA の lib/dll を配置・リンクする必要がありません。
- **柔軟なロードインターフェイス**:
  - 任意のファイルパスからのロード (`load_from_path`)
  - 任意の環境変数からのロード (`load_from_env`)
  - 環境変数優先＋フォールバック指定 (`load_from_env_or_path`)
  - ビルダーパターンによる柔軟な探索設定 (`GnaLibrary::builder()`)
- **安全なRAII管理**:
  - `GnaLibrary` (DLLハンドル)
  - `GnaDevice` (デバイスオープン/クローズ)
  - `GnaBuffer` (GNAアロケータメモリの確保と自動解放)
  - `GnaModel` (コンパイル済みモデルのライフサイクル管理)
- **ゼロフレームワーク対応 (OpenVINO等不要)**:
  - Rustネイティブでモデルを定義・コンパイルする `GnaModelBuilder`
  - 全結合層 (`add_fully_connected_affine` / Dense Layer)
  - モデル構築エラー診断 (`GnaModel::get_last_error_message`)

## 使い方

### 1. 任意のファイルパスからロード

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::load_from_path("C:/path/to/gna.dll")?;
```

### 2. 任意の環境変数からロード

指定した環境変数にDLLのファイルパス、またはDLLが配置されているディレクトリパスを設定してロードできます（ディレクトリの場合は `gna.dll` / `libgna.so` が自動補完されます）。

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::load_from_env("CUSTOM_GNA_LIB_PATH")?;
```

### 3. 環境変数またはフォールバックパス

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::load_from_env_or_path("CUSTOM_GNA_LIB_PATH", "fallback/gna.dll")?;
```

### 4. ビルダーによる優先度付き探索

```rust
use nanai_gna_dll_rs::GnaLibrary;

let lib = GnaLibrary::builder()
    .path("explicit/path/to/gna.dll")      // 1. 指定パス
    .env("MY_APP_GNA_DIR")                 // 2. 指定した環境変数
    .default_envs()                        // 3. 標準の GNA_LIB_PATH, GNA_LIB_DIR
    .current_dir()                         // 4. カレントディレクトリ
    .system_fallback(true)                 // 5. システム検索パス
    .load()?;
```

### 5. デフォルト検索ルールでのロード

```rust
use nanai_gna_dll_rs::GnaLibrary;

// GNA_LIB_PATH -> GNA_LIB_DIR -> カレントディレクトリ -> システムパス の順で探索
let lib = GnaLibrary::load_default()?;
```

### 6. 独自モデルの定義・コンパイル・推論

OpenVINO 等の外部フレームワークを使わず、Rust のみで直接 GNA ネットワークを定義して実行できます。

```rust
use nanai_gna_dll_rs::{
    GnaDevice, GnaLibrary, GnaModelBuilder, Gna2Tensor, Gna2DataType,
    GnaRequestConfig, Gna2AccelerationMode
};

let lib = GnaLibrary::load_default()?;
let device = GnaDevice::open_first(&lib)?;

// テンソル定義 (入力 16x4, 出力 8x4, 重み 8x16, バイアス 8)
let input_t = Gna2Tensor::d2(16, 4, Gna2DataType::Int16, input_buf_ptr);
let output_t = Gna2Tensor::d2(8, 4, Gna2DataType::Int32, output_buf_ptr);
let weight_t = Gna2Tensor::d2(8, 16, Gna2DataType::Int16, weight_buf_ptr);
let bias_t = Gna2Tensor::d1(8, Gna2DataType::Int32, bias_buf_ptr);

// モデル構築
let model = GnaModelBuilder::new()
    .add_fully_connected_affine(input_t, output_t, weight_t, bias_t, None)
    .build(&device)?;

let mut config = GnaRequestConfig::create(&lib, model.id())?;
unsafe {
    config.set_operand_buffer(0, 0, input_buf_ptr)?;
    config.set_operand_buffer(0, 1, output_buf_ptr)?;
}
config.set_acceleration_mode(Gna2AccelerationMode::Auto)?;

// パフォーマンスカウンタ（ハードウェア使用率・サイクル数）の有効化
config.enable_performance_counter()?;

let req_id = config.enqueue()?;
config.wait(req_id, 1000)?;

// ハードウェア使用率 (0.0〜1.0) および詳細統計の取得
let usage = config.get_hw_usage()?;
println!("Hardware usage: {:.2}%", usage * 100.0);

let stats = config.get_performance_stats()?;
println!("Total: {} cycles, Stall: {} cycles", stats.total_cycles, stats.stall_cycles);
```

### 7. 継続的な使用率モニタリング (`GnaUsageMonitor`)

複数回の推論リクエストにわたるハードウェア使用率の移動・累積統計（総稼働サイクル、ストールサイクル、加重平均稼働率）を追跡できます。

```rust
use nanai_gna_dll_rs::GnaUsageMonitor;

let mut monitor = GnaUsageMonitor::new();

// 推論ループ内で記録
for _ in 0..10 {
    let req_id = config.enqueue()?;
    config.wait(req_id, 1000)?;

    if let Ok(stats) = config.get_performance_stats() {
        monitor.record(&stats);
    }
}

println!("推論回数: {}", monitor.inference_count());
println!("累積平均使用率: {:.2}%", monitor.cumulative_hw_usage_percentage());
```

### 8. 負荷テスト・ストレステスト (`GnaLoadTester`)

連続した推論リクエストや指定時間内の高負荷実行（スループット IPS、レイテンシ、キュー深度、HW使用率）を自動計測します。

```rust
use std::time::Duration;
use nanai_gna_dll_rs::{GnaLoadTester, GnaLoadTestConfig};

let config = GnaLoadTestConfig::new()
    .with_iterations(1000)       // 1000回連続実行 (または .with_duration(Duration::from_secs(5)))
    .with_concurrency(2)         // キュー深度（パイプライン同時発行数）
    .with_track_hw_usage(true);  // ハードウェア使用率も集計

let tester = GnaLoadTester::new(config);
let report = tester.run(&mut request_config)?;
report.print_summary();
```

## CLIデモ

```bash
# ヘルプ表示
cargo run --example gna_demo -- --help

# 任意のパスを指定して実行（モデル構築＆推論デモ含む）
cargo run --example gna_demo -- --dll path/to/gna.dll

# 任意の環境変数を指定して実行
cargo run --example gna_demo -- --env MY_GNA_LIB_PATH

# 負荷テスト（1,000回連続推論）
cargo run --example gna_demo -- --stress 1000

# 負荷テスト（5秒間・同時キュー深度2で最大スループット計測）
cargo run --example gna_demo -- --duration 5 --concurrency 2

# デフォルト探索
cargo run --example gna_demo
```

## cli check

```bash
# コードチェック
cargo clippy --fix  --all --allow-dirty

# コード生成
cargo coupling --ai --no-git
```
