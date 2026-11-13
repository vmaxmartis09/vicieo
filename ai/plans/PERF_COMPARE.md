# So sánh hiệu năng backend: Python (socnhotv) vs Rust (vicieo/engine-rust)

Ngày đo: 2026-11-12. Công cụ tải: `ab` (ApacheBench) với `-k -n 200 -c 10`, mỗi endpoint có 1 lượt warm-up 20 request (c=2) trước lượt đo. Chạy lần lượt từng bên, không song song.

## Môi trường
- Apple M4, 10 core, 16 GB RAM, macOS 15.6 (Darwin 24.6), SSD nội bộ; Python 3.15.2, cargo 1.94.0.
- Python: `PYTHONPATH=tool/src .venv/bin/uvicorn socnhotv.core.api.app:app --port 8000 --reload --reload-dir tool/src` (đúng lệnh `socrun api` trong `scripts/socrun.mjs`; mặc định có `--reload`, cwd `/Volumes/Data/socnhotv`).
- Rust: `cargo build --release` rồi `cargo run --release -- serve --port 9001` (cwd `engine-rust`). **Cổng 9001, không phải 9000**: cổng 9000 đang có một server dev (`target/debug/vicieo`) của người dùng và tôi không được phép tắt nó. Server đó và `next dev -p 3600` luôn chạy nền ở trạng thái rảnh trong cả hai lần đo (ảnh hưởng nhỏ, như nhau).

## Dữ liệu đo
- Đọc trên `gobi` thật (chỉ đọc) và trên project tạm `zz-bench` tạo **giống hệt** ở cả hai repo (graph.json copy từ gobi của vicieo, 3 product, danh sách nhân vật, ảnh 2 MB và video 20 MB ngẫu nhiên). Ghi chỉ thực hiện trên `zz-bench`; đã xoá sạch sau đo (đã kiểm tra `data/projects` chỉ còn các project gốc). Không chạm gobi/soc-nho-tv, không chạy graph có provider, không dùng API key.
- Dữ liệu gobi của hai repo không giống nhau (graph 18.7 KB vs 20.0 KB; thư mục Python có thêm `runs/`, `output/`).

## Khởi động và tài nguyên
| Chỉ số | Python | Rust |
|---|---|---|
| Từ lệnh đến /health 200 | 0.682 s (lần đo đầu: 1.31 s, cold) | 0.775 s (đã build sẵn; gồm `cargo run` kiểm tra) |
| RAM idle (RSS) | 156.1 MB (cây tiến trình reloader + worker) | 10.7 MB |
| CPU idle (10 s) | 5.1 % (watcher của `--reload`) | 0.0 % |
| RAM sau tải | 163.3 MB | 277.0 MB (không giảm sau 10 s) |
| CPU tích luỹ trong lúc tải + 10 s | 6.73 s | 5.3 s |

## Độ trễ và throughput (200 req, c=10)
Ô số: p50 ms / p95 ms / req/s. `ab` chỉ có độ phân giải 1 ms nên p50 = 0 nghĩa là < 1 ms. Độ dài = kích thước body trả về (byte). Các dòng `write_*` dùng c=1, n=50 (tránh race ghi cùng file).

| Endpoint (khoá) | Py bytes | Python | Rust bytes | Rust | Rust/Py (req/s) |
|---|---|---|---|---|---|
| health | 138 | 4 / 5 / 2,386 | 182 | 0 / 0 / 60,976 | 25.6× |
| projects | 753 | 4 / 6 / 2,268 | 11191 | 1 / 3 / 6,154 | 2.7× |
| graph_gobi | 15758 | 5 / 6 / 1,947 | 16722 | 0 / 1 / 33,228 | 17.1× |
| graph_bench | 16606 | 5 / 5 / 2,025 | 16722 | 0 / 0 / 35,379 | 17.5× |
| characters_gobi | 1039 | 4 / 5 / 2,347 | 14501 | 0 / 1 / 26,038 | 11.1× |
| characters_bench | 14501 | 7 / 7 / 1,448 | 16801 | 0 / 1 / 21,342 | 14.7× |
| products_gobi | 16240 | 6 / 6 / 1,622 | 16240 | 0 / 1 / 21,406 | 13.2× |
| products_bench | 2 | 3 / 3 / 3,760 | 2 | 0 / 0 / 50,176 | 13.3× |
| node_types | 121272 | 6 / 6 / 1,726 | 121272 | 2 / 5 / 3,748 | 2.2× |
| models | 20880 | 9 / 10 / 1,068 | 20295 | 0 / 1 / 22,758 | 21.3× |
| providers | 3377 | 7 / 7 / 1,450 | 840 | 0 / 0 / 69,759 | 48.1× |
| file_img_2MB | 2097152 | 20 / 21 / 491 | 2097152 | 2 / 5 / 3,858 | 7.9× |
| file_vid_20MB | 20971520 | 146 / 161 / 68 | 20971520 | 29 / 60 / 317 | 4.6× |
| validate_post | 256 | 4 / 5 / 2,167 | 269 | 9 / 16 / 1,013 | 0.5× |
| write_character | 76 | 1 / 1 / 1,020 | 45 | 0 / 1 / 2,981 | 2.9× |
| write_graph | 59 | 1 / 1 / 1,125 | 92 | 0 / 1 / 3,467 | 3.1× |

Ánh xạ endpoint: health `/health`; projects Py `/projects` ↔ Rust `/data/projects`; graph `/projects/{id}/graph`; characters Py `/projects/{id}/characters` ↔ Rust `/data/projects/{id}/characters`; products `/projects/{id}/products`; `/node-types`, `/models`, `/providers`; file `/projects/zz-bench/files/assets/{img.png,vid.mp4}`; validate `POST /graph/validate`; write_character `POST characters`, write_graph `POST /projects/zz-bench/graph`. TTFB: `ab` không tách TTFB chính xác; cột "Waiting" cho file 20 MB: Python ~3 ms, Rust ~15 ms (xem dưới).

## Điểm Rust chưa tương đương / lưu ý
- Payload khác nhau nên so sánh chỉ mang tính chỉ báo: `/health` 182 vs 138 B; danh sách project 11.2 KB (Rust) vs 0.75 KB (Py); `/providers` 0.84 KB vs 3.4 KB (Rust trả rút gọn); `/models` 20.3 vs 20.9 KB; characters gobi 14.5 KB vs 1.0 KB (khác dữ liệu/định dạng); `validate` 269 vs 256 B.
- Phục vụ file: Rust `fs::read` cả file vào RAM rồi mới trả (không stream) → TTFB ảnh/video lớn tệ hơn Python (`FileResponse` stream), và RSS tăng 10.7 → 277 MB sau tải 20 MB × 10 kết nối, không giải phóng được trong 10 s.
- Rust có các nhánh mock (`mockMode` mặc định true) ở `ai/character`, batch, chat, provider `mock`; không đo các đường này.
- Định dạng `studio_characters.json` khác nhau: Rust dùng `{"characters":[...]}`, Python dùng mảng thuần (fixture Python phải đổi sang mảng, nếu không POST trả 500).
- Python chạy mặc định với `--reload` nên RAM/CPU idle gồm tiến trình watcher; chạy production không reload sẽ thấp hơn (không đo).
- Không đo: SSE `/events`, run graph, render video, các endpoint gọi provider.

## Kết luận
**Đã cải thiện (đáng tin):**
- RAM idle: ~156 MB → ~11 MB (≈14× nhỏ hơn); CPU idle 5.1 % → 0 %.
- Throughput các endpoint đọc nhỏ/JSON: health, graph, characters, products, models, providers gấp ~11–48× (xem bảng; danh sách projects chỉ 2.7×), p50/p95 từ 4–9 ms xuống <1–3 ms.
- Ghi (character, graph) trên project tạm: throughput ~3× (c=1).
- Ảnh 2 MB: ~8× throughput (3.9k vs 0.49k req/s).

**Chưa cải thiện / tệ hơn:**
- (Đã sửa, xem Phụ lục) `POST /graph/validate`: Rust ~1.0k req/s, p50 9 ms, **chậm hơn Python** (~2.2k req/s, p50 4 ms). Chưa điều tra nguyên nhân (có thể validator Rust làm nhiều việc hơn).
- Video 20 MB: tổng thời gian nhanh hơn ~4.6× về throughput nhưng TTFB tệ hơn (~15 ms vs ~3 ms) và p95 thấp hơn nhiều; RAM tăng mạnh vì không stream.
- Khởi động: Rust 0.78 s vs Python 0.68 s trong lần đo thứ hai (Python lần đầu 1.31 s) — khởi động Rust đã gồm bước `cargo run`; **không có cải thiện rõ**.
- `/node-types`: ~2× (payload 121 KB giống nhau), ít hơn các endpoint khác.

**Số liệu kém tin cậy:**
- Mọi endpoint có payload khác nhau (xem trên) và dữ liệu gobi khác nhau; chỉ `graph_bench`, `products_bench`, file và `node-types` (cùng 121 272 B) gần như cùng điều kiện.
- Đọc từ cache OS (file nhỏ, cùng một đường dẫn lặp lại); tất cả 200 request là cùng URL nên Rust/Python có thể cache hit ở page cache.
- `ab` làm tròn 1 ms, 200 request chỉ ~vài chục ms mỗi lượt; p95 ở mức vài ms dao động giữa các lần chạy (chỉ chạy mỗi bên 1 lần, không lặp lại để lấy trung vị).
- Startup Python giữa 2 lần đo chênh gấp đôi (cache bytecode); startup Rust chỉ đo 1 lần.
- Rust chạy trên cổng 9001 trong khi một server Rust debug khác nằm rảnh ở 9000.

## Phụ lục: sửa validate và đo lại
Nguyên nhân: `validator::node_types()` đọc + parse `node_types.json` (193 KB) và clone từng định nghĩa trong mỗi request. Đã sửa (engine-rust/src/engine/validator.rs, routes/graph.rs):
1. Cache registry trong `Arc<HashMap>` (`once_cell::Lazy` + `Mutex`), nạp lại khi mtime file đổi (1 lần `stat` mỗi call).
2. `Regex` `$node.<id>` biên dịch một lần (`Lazy`).
3. `effective()` trả `Cow` — chỉ clone khi capability được bật.
4. Bỏ clone mảng `nodes` và clone body trong handler.
5. Thêm test `registry_is_cached_between_calls`; 7 test validator đều pass. Body trả về không đổi (269 B).

Đo lại cùng harness, cổng 9001 (số liệu trước/sau, p50/p95 ms / req/s):

| Endpoint | Rust trước | Rust sau | Python |
|---|---|---|---|
| validate_post | 9 / 16 / 1.013 | 0 / 1 / 16.519 | 4 / 5 / 2.167 |
| write_graph (c=1) | 0 / 1 / 3.467 | 0 / 0 / 6.575 | 1 / 1 / 1.125 |
| node_types | 2 / 5 / 3.748 | 1 / 4 / 4.839 | 6 / 6 / 1.726 |
| file_vid_20MB | 29 / 60 / 317 | 19 / 42 / 443 | 146 / 161 / 68 |

validate nhanh hơn ~16× so với trước và ~7,6× so với Python; `write_graph` cũng hưởng lợi (cùng đường validate). Các endpoint khác chỉ dao động trong nhiễu đo (chạy 1 lần).
Lưu ý: số "startup 18,9 s" của lần đo này là do `cargo run` biên dịch lại sau khi tôi thêm test, không phải thời gian khởi động server; RSS sau tải lần này 44 MB (trước 277 MB) — chênh lớn nhưng chưa rõ nguyên nhân (có thể do allocator), không kết luận. Chưa làm: stream file (TTFB video), kiểm tra riêng `/node-types`.
