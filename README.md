# microPixdown
[ドットアニメーションに特化したラスタ画像軽量マークアップ言語](https://github.com/Ablaze-MIRAI/pixdown)のマイコンLED向けエディション

![Crates.io Version](https://img.shields.io/crates/v/micro-pixdown?style=flat&logo=rust)

## How to write
microPixdownのデータ構造はヘッダーと内容の2つに分けられます
```
---
(ヘッダ部分)
---
(内容)
```

### ヘッダー
```toml
mix = "!(0 ^ 1)" # 論理式でレイヤー合成を書く

[size]
w = 2 # 幅
h = 2 # 高さ
frames = 8 # フレームの数
rate = [1, 4] # フレーム間隔 ([0]/[1] s)

[binaries] # 点灯/消灯の定義
"0" = false
"1" = true

[options] # オプション(なくてもよい)
order = [1, 0, 1, 0, 0, 1, 0, 0] # 順序指定
```

### 内容
```md
# 0
10
01

# 1
## 0
10
01

## 1
01
10
```
`#`: レイヤー番号

`##`: フレーム番号

### 出力結果(整形済み)
```json
{
  "width": 2,
  "height": 2,
  "rate": [1, 4],
  "frames": [
    [
      [false, false],
      [false, false]
    ],
    [
      [true, true],
      [true, true]
    ],
    [
      [false, false],
      [false, false]
    ],
    [
      [true, true],
      [true, true]
    ],
    [
      [true, true],
      [true, true]
    ],
    [
      [false, false],
      [false, false]
    ],
    [
      [true, true],
      [true, true]
    ],
    [
      [true, true],
      [true, true]
    ]
  ]
}
```

## How to use
### Rust project
#### Install
```sh
cargo add micro-pixdown
```

#### Use
```rust
use micro_pixdown::compile;
use std::fs::{File, read_to_string};
use std::io::Write;

fn main() {
    let text = read_to_string("example.mpxd").unwrap();
    let b = compile(&text) {
    let mut file = File::create("example.json").unwrap();
    file.write_all(b.as_bytes()).unwrap();
    file.flush().unwrap();
}
```

## Demo
リファレンス実装が動かせます
```sh
cargo run -- [Pixdownファイル] [出力先]
```

## Donation
私に寄付するくらいなら私のファンアートを描くか私の曲を聴くかしてください

それでも寄付したい人はAblazeに寄付してあげてください

## License
[Rion Hobby License](LICENSE) (ほぼISC)で公開しています
